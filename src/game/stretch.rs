use rodio::{ChannelCount, Sample, SampleRate, Source};
use std::{
    sync::{
        atomic::{AtomicU32, Ordering},
        Arc,
    },
    time::Duration,
};

const GRAIN_TIME: f32 = 0.04; // seconds per grain, overlapped by half
const SEEK_TIME: f32 = 0.012; // seconds searched each side for the best splice
const COARSE: usize = 4; // coarse search step, frames
const TIE: f32 = 1e-3; // relative score margin favoring the continuation
const MIN_TEMPO: f32 = 0.5;
const MAX_TEMPO: f32 = 2.0;

/// tempo ratio shared with a playing stretch
#[derive(Clone)]
pub struct Tempo(Arc<AtomicU32>);

impl Tempo {
    pub fn new() -> Self {
        Tempo(Arc::new(AtomicU32::new(1.0f32.to_bits())))
    }

    pub fn set(&self, tempo: f32) {
        self.0.store(tempo.to_bits(), Ordering::Relaxed);
    }

    fn get(&self) -> f32 {
        f32::from_bits(self.0.load(Ordering::Relaxed)).clamp(MIN_TEMPO, MAX_TEMPO)
    }
}

/// wsola time stretch, tempo without pitch
pub struct Stretch<S> {
    source: S,
    tempo: Tempo,
    channels: ChannelCount,
    sample_rate: SampleRate,
    grain: usize, // frames
    hop: usize,   // frames
    seek: usize,  // frames
    window: Vec<f32>,
    input: Vec<Sample>, // interleaved, from frame base
    base: usize,
    ended: bool,         // source exhausted
    pos: f64,            // nominal input frame of the next grain
    next: Option<usize>, // natural continuation of the last grain
    out: Vec<Sample>,    // overlap add, one grain
    ready: usize,        // samples ready at the front of out
    cursor: usize,
    flushed: bool,
}

impl<S: Source> Stretch<S> {
    pub fn new(source: S, tempo: Tempo) -> Self {
        let channels = source.channels();
        let sample_rate = source.sample_rate();
        let rate = sample_rate.get() as f32;
        let hop = ((rate * GRAIN_TIME) as usize / 2).max(1);
        let grain = hop * 2;
        let seek = (rate * SEEK_TIME) as usize;
        // periodic hann, sums to one at half overlap
        let window = (0..grain)
            .map(|i| 0.5 - 0.5 * (std::f32::consts::TAU * i as f32 / grain as f32).cos())
            .collect();
        let ch = channels.get() as usize;
        Stretch {
            source,
            tempo,
            channels,
            sample_rate,
            grain,
            hop,
            seek,
            window,
            input: Vec::new(),
            base: 0,
            ended: false,
            pos: 0.0,
            next: None,
            out: vec![0.0; grain * ch],
            ready: 0,
            cursor: 0,
            flushed: false,
        }
    }

    fn ch(&self) -> usize {
        self.channels.get() as usize
    }

    fn end_frame(&self) -> usize {
        self.base + self.input.len() / self.ch()
    }

    /// decode up to frame `to`
    fn fill(&mut self, to: usize) {
        let ch = self.ch();
        while !self.ended && self.end_frame() < to {
            match self.source.next() {
                Some(sample) => self.input.push(sample),
                None => {
                    self.ended = true;
                    let partial = self.input.len() % ch;
                    if partial != 0 {
                        self.input.resize(self.input.len() + ch - partial, 0.0);
                    }
                }
            }
        }
    }

    fn sample(&self, frame: usize, channel: usize) -> Sample {
        frame
            .checked_sub(self.base)
            .and_then(|f| self.input.get(f * self.ch() + channel))
            .copied()
            .unwrap_or(0.0)
    }

    fn mono(&self, frame: usize) -> f32 {
        (0..self.ch()).map(|c| self.sample(frame, c)).sum()
    }

    /// start in lo..=hi best matching the continuation at target
    fn best(&self, lo: usize, hi: usize, target: usize) -> usize {
        let len = self.hop;
        let span: Vec<f32> = (lo..hi + len).map(|f| self.mono(f)).collect();
        let goal: Vec<f32> = (target..target + len).map(|f| self.mono(f)).collect();
        let score = |k: usize, step: usize| {
            let off = k - lo;
            let (mut num, mut den) = (0.0, 0.0);
            for i in (0..len).step_by(step) {
                let b = span[off + i];
                num += goal[i] * b;
                den += b * b;
            }
            num / (den + 1e-9f32).sqrt()
        };
        let pick = |candidates: &mut dyn Iterator<Item = usize>, step: usize| {
            candidates
                .map(|k| (k, score(k, step)))
                .fold((lo, f32::MIN), |a, b| if b.1 > a.1 { b } else { a })
                .0
        };
        let coarse = pick(&mut (lo..=hi).step_by(COARSE), COARSE);
        let near = coarse.saturating_sub(COARSE - 1).max(lo)..=(coarse + COARSE - 1).min(hi);
        let found = pick(&mut near.into_iter(), 2);
        // prefer the continuation on near ties, keeps tempo 1 lossless
        if (lo..=hi).contains(&target) && score(target, 2) >= score(found, 2) * (1.0 - TIE) {
            target
        } else {
            found
        }
    }

    /// overlap add the next grain, false at the end
    fn grain(&mut self) -> bool {
        let nominal = self.pos.round() as usize;
        let lo = nominal.saturating_sub(self.seek).max(self.base);
        let hi = nominal + self.seek;
        let need = (hi + self.grain).max(self.next.map_or(0, |t| t + self.hop));
        self.fill(need);
        if self.ended && nominal >= self.end_frame() {
            return false;
        }
        let start = match self.next {
            Some(target) => self.best(lo, hi, target),
            None => nominal,
        };
        let ch = self.ch();
        for i in 0..self.grain {
            let w = self.window[i];
            for c in 0..ch {
                self.out[i * ch + c] += w * self.sample(start + i, c);
            }
        }
        self.next = Some(start + self.hop);
        self.pos += self.hop as f64 * self.tempo.get() as f64;

        // drop input no longer reachable
        let keep = (self.pos as usize)
            .saturating_sub(self.seek)
            .min(start + self.hop)
            .min(self.end_frame());
        if keep > self.base + self.grain {
            self.input.drain(..(keep - self.base) * ch);
            self.base = keep;
        }
        true
    }
}

impl<S: Source> Iterator for Stretch<S> {
    type Item = Sample;

    fn next(&mut self) -> Option<Sample> {
        if self.cursor == self.ready {
            let ch = self.ch();
            let hop = self.hop * ch;
            self.out.copy_within(self.ready.., 0);
            let len = self.out.len();
            self.out[len - self.ready..].fill(0.0);
            self.cursor = 0;
            if self.grain() {
                self.ready = hop;
            } else if !self.flushed {
                // fading tail of the last grain
                self.flushed = true;
                self.ready = hop;
            } else {
                return None;
            }
        }
        let sample = self.out[self.cursor];
        self.cursor += 1;
        Some(sample)
    }
}

impl<S: Source> Source for Stretch<S> {
    fn current_span_len(&self) -> Option<usize> {
        None
    }

    fn channels(&self) -> ChannelCount {
        self.channels
    }

    fn sample_rate(&self) -> SampleRate {
        self.sample_rate
    }

    fn total_duration(&self) -> Option<Duration> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rodio::buffer::SamplesBuffer;
    use std::num::NonZero;

    fn tone(frames: usize) -> SamplesBuffer {
        let samples: Vec<f32> = (0..frames)
            .flat_map(|i| {
                let s = (i as f32 * 0.05).sin() * 0.5;
                [s, -s]
            })
            .collect();
        SamplesBuffer::new(
            NonZero::new(2).unwrap(),
            NonZero::new(44_100).unwrap(),
            samples,
        )
    }

    #[test]
    fn unit_tempo_reconstructs() {
        let original: Vec<f32> = tone(20_000).collect();
        let out: Vec<f32> = Stretch::new(tone(20_000), Tempo::new()).collect();
        let hop = 882 * 2;
        // the first half grain fades in
        for (i, (a, b)) in original.iter().zip(&out).enumerate().skip(hop) {
            assert!((a - b).abs() < 1e-4, "sample {i}: {a} vs {b}");
        }
    }

    #[test]
    fn fast_tempo_reaches_end() {
        let tempo = Tempo::new();
        tempo.set(MAX_TEMPO);
        assert!(Stretch::new(tone(10_007), tempo).count() > 0);
    }

    #[test]
    fn faster_tempo_shortens() {
        let tempo = Tempo::new();
        tempo.set(1.25);
        let out = Stretch::new(tone(44_100), tempo).count() / 2;
        let expected = (44_100.0 / 1.25) as usize;
        assert!(
            out.abs_diff(expected) < 2_000,
            "{out} frames, expected {expected}"
        );
    }
}
