//! Synthesized kick drum: a sine wave whose pitch sweeps down fast while its volume decays.

use std::f32::consts::TAU;

/// Pitch at the moment of the hit.
const START_HZ: f32 = 150.0;
/// Pitch the sweep settles toward.
const END_HZ: f32 = 50.0;
/// Time constant of the pitch sweep (how fast 150 Hz falls toward 50 Hz).
const PITCH_TAU_SECONDS: f32 = 0.03;
/// Time constant of the volume decay.
const AMP_TAU_SECONDS: f32 = 0.08;
/// Total length of one hit. After this the kick is silent.
const LENGTH_SECONDS: f32 = 0.4;
/// Output level, kept below 1.0 to leave headroom.
const GAIN: f32 = 0.8;

pub struct Kick {
    sample_rate: f32,
    /// Per-sample multipliers that turn the exponential decays into one multiply each.
    pitch_coeff: f32,
    amp_coeff: f32,
    length_samples: u32,

    // State of the current hit.
    phase: f32,
    pitch_offset_hz: f32,
    amp: f32,
    remaining_samples: u32,
}

impl Kick {
    /// A kick with no sample rate yet. It stays silent until `set_sample_rate` is called.
    pub const fn new() -> Self {
        Self {
            sample_rate: 0.0,
            pitch_coeff: 0.0,
            amp_coeff: 0.0,
            length_samples: 0,
            phase: 0.0,
            pitch_offset_hz: 0.0,
            amp: 0.0,
            remaining_samples: 0,
        }
    }

    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        // Rejects zero, negatives, NaN, and infinity.
        if !sample_rate.is_finite() || sample_rate <= 0.0 {
            return;
        }
        self.sample_rate = sample_rate;
        self.pitch_coeff = decay_coeff(PITCH_TAU_SECONDS, sample_rate);
        self.amp_coeff = decay_coeff(AMP_TAU_SECONDS, sample_rate);
        self.length_samples = (LENGTH_SECONDS * sample_rate) as u32;
        self.remaining_samples = 0;
    }

    /// Start a hit from the beginning, cutting off any hit already playing.
    pub fn trigger(&mut self) {
        if self.sample_rate <= 0.0 {
            return;
        }
        self.phase = 0.0;
        self.pitch_offset_hz = START_HZ - END_HZ;
        self.amp = 1.0;
        self.remaining_samples = self.length_samples;
    }

    /// Overwrite `out` with the next samples of the kick (zeros when nothing is playing).
    pub fn render(&mut self, out: &mut [f32]) {
        for sample in out.iter_mut() {
            *sample = self.next_sample();
        }
    }

    fn next_sample(&mut self) -> f32 {
        if self.remaining_samples == 0 {
            return 0.0;
        }
        let value = self.phase.sin() * self.amp * GAIN;

        let freq_hz = END_HZ + self.pitch_offset_hz;
        self.phase = (self.phase + TAU * freq_hz / self.sample_rate) % TAU;
        self.pitch_offset_hz *= self.pitch_coeff;
        self.amp *= self.amp_coeff;
        self.remaining_samples -= 1;

        value
    }
}

/// Multiplier that shrinks a value by a factor of e every `tau_seconds`.
fn decay_coeff(tau_seconds: f32, sample_rate: f32) -> f32 {
    (-1.0 / (tau_seconds * sample_rate)).exp()
}

#[cfg(test)]
mod tests {
    use super::*;

    const RATE: f32 = 48_000.0;
    const BLOCK: usize = 128;

    fn ready_kick() -> Kick {
        let mut kick = Kick::new();
        kick.set_sample_rate(RATE);
        kick
    }

    fn render_seconds(kick: &mut Kick, seconds: f32) -> Vec<f32> {
        let mut out = vec![0.0; (seconds * RATE) as usize];
        kick.render(&mut out);
        out
    }

    #[test]
    fn is_silent_before_a_trigger() {
        let mut kick = ready_kick();
        let out = render_seconds(&mut kick, 0.1);
        assert!(out.iter().all(|&s| s == 0.0));
    }

    #[test]
    fn makes_sound_after_a_trigger() {
        let mut kick = ready_kick();
        kick.trigger();
        let out = render_seconds(&mut kick, 0.05);
        let peak = out.iter().fold(0.0f32, |m, s| m.max(s.abs()));
        assert!(peak > 0.5, "peak was {peak}");
    }

    #[test]
    fn stays_within_unit_range() {
        let mut kick = ready_kick();
        kick.trigger();
        let out = render_seconds(&mut kick, LENGTH_SECONDS);
        assert!(out.iter().all(|s| s.abs() <= 1.0));
    }

    #[test]
    fn decays_and_then_goes_silent() {
        let mut kick = ready_kick();
        kick.trigger();
        let out = render_seconds(&mut kick, LENGTH_SECONDS + 0.1);
        let length = (LENGTH_SECONDS * RATE) as usize;

        let tail_peak = out[length - BLOCK..length]
            .iter()
            .fold(0.0f32, |m, s| m.max(s.abs()));
        assert!(tail_peak < 0.01, "tail peak was {tail_peak}");
        assert!(out[length..].iter().all(|&s| s == 0.0));
    }

    #[test]
    fn retrigger_restarts_from_the_beginning() {
        let mut fresh = ready_kick();
        fresh.trigger();
        let mut expected = [0.0; BLOCK];
        fresh.render(&mut expected);

        let mut kick = ready_kick();
        kick.trigger();
        render_seconds(&mut kick, 0.1);
        kick.trigger();
        let mut actual = [0.0; BLOCK];
        kick.render(&mut actual);

        assert_eq!(actual, expected);
    }

    #[test]
    fn stays_silent_without_a_sample_rate() {
        let mut kick = Kick::new();
        kick.trigger();
        let mut out = [1.0; BLOCK];
        kick.render(&mut out);
        assert!(out.iter().all(|&s| s == 0.0));
    }

    #[test]
    fn ignores_an_invalid_sample_rate() {
        let mut kick = Kick::new();
        kick.set_sample_rate(0.0);
        kick.set_sample_rate(f32::NAN);
        kick.trigger();
        let mut out = [1.0; BLOCK];
        kick.render(&mut out);
        assert!(out.iter().all(|&s| s == 0.0));
    }
}
