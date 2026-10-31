//! The drill's sound: effects and a looping march, synthesised with Web Audio so no recording ships.

use leptos::prelude::*;
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::time::Duration;
use web_sys::{
    AudioBuffer, AudioContext, AudioNode, AudioScheduledSourceNode, BiquadFilterType, GainNode, OscillatorType,
};

const MUSIC_VOLUME: f32 = 0.3;
const EFFECTS_VOLUME: f32 = 0.55;
/// The march's length in sixteenth notes: four bars.
const LOOP: usize = 64;
/// Seconds of music scheduled ahead of the clock, so timer jitter never gaps it.
const AHEAD: f64 = 0.12;

thread_local! {
    static AUDIO: RefCell<Option<Rc<Audio>>> = const { RefCell::new(None) };
}

pub struct Audio {
    ctx: AudioContext,
    music: GainNode,
    effects: GainNode,
    noise: AudioBuffer,
    pub tempo: Cell<f64>,
}

/// The shared engine, made on first use. Browsers keep it silent until `wake` runs from a press.
pub fn audio() -> Option<Rc<Audio>> {
    AUDIO.with_borrow_mut(|a| {
        if a.is_none() {
            *a = Audio::new().ok().map(Rc::new);
        }
        a.clone()
    })
}

/// The engine, if a press has already made it.
pub fn current() -> Option<Rc<Audio>> {
    AUDIO.with_borrow(|a| a.clone())
}

fn freq(midi: f64) -> f64 {
    440.0 * 2f64.powf((midi - 69.0) / 12.0)
}

struct Voice {
    wave: OscillatorType,
    vol: f32,
    lowpass: Option<f32>,
}

const SQUARE: Voice = Voice { wave: OscillatorType::Square, vol: 0.22, lowpass: None };
const TRIANGLE: Voice = Voice { wave: OscillatorType::Triangle, vol: 0.3, lowpass: None };

impl Audio {
    fn new() -> Result<Audio, wasm_bindgen::JsValue> {
        let ctx = AudioContext::new()?;
        let master = ctx.create_gain()?;
        master.gain().set_value(0.8);
        master.connect_with_audio_node(&ctx.destination())?;
        let music = ctx.create_gain()?;
        music.connect_with_audio_node(&master)?;
        let effects = ctx.create_gain()?;
        effects.connect_with_audio_node(&master)?;
        let rate = ctx.sample_rate();
        let noise = ctx.create_buffer(1, rate as u32, rate)?;
        let mut seed = 0x2545_f491u32;
        let samples: Vec<f32> = (0..rate as u32)
            .map(|_| {
                seed ^= seed << 13;
                seed ^= seed >> 17;
                seed ^= seed << 5;
                seed as f32 / u32::MAX as f32 * 2.0 - 1.0
            })
            .collect();
        noise.copy_to_channel(&samples, 0)?;
        Ok(Audio { ctx, music, effects, noise, tempo: Cell::new(140.0) })
    }

    pub fn wake(&self) {
        let _ = self.ctx.resume();
    }

    pub fn set_levels(&self, music: bool, effects: bool) {
        let t = self.now();
        let set = |g: &GainNode, v: f32| {
            let p = g.gain();
            let _ = p.cancel_scheduled_values(t);
            let _ = p.set_target_at_time(v, t, 0.05);
        };
        set(&self.music, if music { MUSIC_VOLUME } else { 0.0 });
        set(&self.effects, if effects { EFFECTS_VOLUME } else { 0.0 });
    }

    fn now(&self) -> f64 {
        self.ctx.current_time()
    }

    fn tone(&self, out: &GainNode, v: &Voice, hz: f64, at: f64, dur: f64, glide_to: Option<f64>) {
        let Ok(osc) = self.ctx.create_oscillator() else { return };
        let Ok(env) = self.ctx.create_gain() else { return };
        osc.set_type(v.wave);
        let f = osc.frequency();
        let _ = f.set_value_at_time(hz as f32, at);
        if let Some(to) = glide_to {
            let _ = f.exponential_ramp_to_value_at_time(to as f32, at + dur);
        }
        let g = env.gain();
        let _ = g.set_value_at_time(0.0, at);
        let _ = g.linear_ramp_to_value_at_time(v.vol, at + 0.004);
        let _ = g.exponential_ramp_to_value_at_time(0.0001, at + dur);
        let head: AudioNode = match v.lowpass.and_then(|cut| self.filter(BiquadFilterType::Lowpass, cut, &env)) {
            Some(filter) => filter.into(),
            None => env.clone().into(),
        };
        let _ = osc.connect_with_audio_node(&head);
        let _ = env.connect_with_audio_node(out);
        let _ = osc.start_with_when(at);
        let _ = osc.stop_with_when(at + dur + 0.02);
    }

    fn filter(&self, kind: BiquadFilterType, hz: f32, into: &GainNode) -> Option<web_sys::BiquadFilterNode> {
        let f = self.ctx.create_biquad_filter().ok()?;
        f.set_type(kind);
        f.frequency().set_value(hz);
        f.connect_with_audio_node(into).ok()?;
        Some(f)
    }

    fn hiss(&self, out: &GainNode, at: f64, dur: f64, vol: f32, kind: BiquadFilterType, hz: f32) {
        let Ok(src) = self.ctx.create_buffer_source() else { return };
        let Ok(env) = self.ctx.create_gain() else { return };
        let Some(filter) = self.filter(kind, hz, &env) else { return };
        src.set_buffer(Some(&self.noise));
        let g = env.gain();
        let _ = g.set_value_at_time(vol, at);
        let _ = g.exponential_ramp_to_value_at_time(0.0001, at + dur);
        let _ = src.connect_with_audio_node(&filter);
        let _ = env.connect_with_audio_node(out);
        let src: &AudioScheduledSourceNode = &src;
        let _ = src.start_with_when(at);
        let _ = src.stop_with_when(at + dur + 0.02);
    }

    fn kick(&self, at: f64, vol: f32) {
        let v = Voice { wave: OscillatorType::Sine, vol, lowpass: None };
        self.tone(&self.music, &v, 150.0, at, 0.22, Some(42.0));
    }

    fn snare(&self, at: f64, vol: f32) {
        self.hiss(&self.music, at, 0.12, vol, BiquadFilterType::Highpass, 1600.0);
        self.tone(&self.music, &Voice { vol: vol * 0.5, ..TRIANGLE }, 190.0, at, 0.07, None);
    }

    fn hat(&self, at: f64) {
        self.hiss(&self.music, at, 0.03, 0.06, BiquadFilterType::Highpass, 7500.0);
    }
}

// ---------- Effects ----------

/// A correct arrow: each one a step up the pentatonic scale.
pub fn key(a: &Audio, step: usize) {
    const SCALE: [f64; 9] = [72.0, 74.0, 76.0, 79.0, 81.0, 84.0, 86.0, 88.0, 91.0];
    let t = a.now();
    let m = SCALE[step.min(SCALE.len() - 1)];
    a.tone(&a.effects, &Voice { vol: 0.16, ..SQUARE }, freq(m), t, 0.07, None);
    a.tone(&a.effects, &Voice { vol: 0.12, ..TRIANGLE }, freq(m - 12.0), t, 0.09, None);
}

pub fn miss(a: &Audio) {
    let t = a.now();
    let buzz = Voice { wave: OscillatorType::Sawtooth, vol: 0.28, lowpass: Some(1200.0) };
    a.tone(&a.effects, &buzz, 150.0, t, 0.24, Some(70.0));
    a.hiss(&a.effects, t, 0.08, 0.12, BiquadFilterType::Bandpass, 900.0);
}

pub fn called(a: &Audio) {
    let t = a.now();
    for (i, m) in [84.0, 88.0, 91.0].into_iter().enumerate() {
        let at = t + i as f64 * 0.045;
        a.tone(&a.effects, &Voice { vol: 0.14, ..SQUARE }, freq(m), at, 0.12, None);
        a.tone(&a.effects, &TRIANGLE, freq(m), at, 0.16, None);
    }
}

pub fn cleared(a: &Audio) {
    let t = a.now() + 0.05;
    for (i, m) in [72.0, 76.0, 79.0, 84.0].into_iter().enumerate() {
        let at = t + i as f64 * 0.1;
        let dur = if i == 3 { 0.55 } else { 0.12 };
        a.tone(&a.effects, &Voice { vol: 0.16, ..SQUARE }, freq(m), at, dur, None);
        a.tone(&a.effects, &TRIANGLE, freq(m - 12.0), at, dur, None);
    }
    for i in 0..6 {
        a.hiss(&a.effects, t + i as f64 * 0.05, 0.05, 0.05 + i as f32 * 0.02, BiquadFilterType::Highpass, 1800.0);
    }
}

/// The countdown's ticks, and its higher note on the go.
pub fn count(a: &Audio, go: bool) {
    let t = a.now();
    let (hz, dur) = if go { (1320.0, 0.3) } else { (880.0, 0.09) };
    a.tone(&a.effects, &Voice { vol: 0.18, ..SQUARE }, hz, t, dur, None);
}

/// The fuse running low.
pub fn alarm(a: &Audio) {
    a.tone(&a.effects, &Voice { vol: 0.1, ..SQUARE }, 1046.5, a.now(), 0.05, None);
}

pub fn over(a: &Audio) {
    let t = a.now();
    let horn = Voice { wave: OscillatorType::Sawtooth, vol: 0.2, lowpass: Some(1800.0) };
    for (i, m) in [76.0, 72.0, 67.0, 60.0].into_iter().enumerate() {
        let at = t + i as f64 * 0.18;
        a.tone(&a.effects, &horn, freq(m), at, if i == 3 { 0.7 } else { 0.18 }, None);
    }
    a.hiss(&a.effects, t, 0.5, 0.2, BiquadFilterType::Lowpass, 400.0);
}

pub fn best(a: &Audio) {
    let t = a.now() + 0.8;
    for (i, m) in [88.0, 91.0, 96.0, 100.0].into_iter().enumerate() {
        a.tone(&a.effects, &Voice { vol: 0.12, ..TRIANGLE }, freq(m), t + i as f64 * 0.07, 0.25, None);
    }
}

// ---------- Music ----------

/// Bar by bar: the chord's root for the bass, and its tones for the stab.
const BARS: [(f64, [f64; 3]); 4] =
    [(45.0, [57.0, 60.0, 64.0]), (41.0, [53.0, 57.0, 60.0]), (48.0, [55.0, 60.0, 64.0]), (43.0, [55.0, 59.0, 62.0])];
/// The bass line within a bar, in semitones over the root.
const BASS: [Option<f64>; 16] = [
    Some(0.0),
    None,
    Some(0.0),
    None,
    Some(12.0),
    None,
    Some(0.0),
    None,
    Some(0.0),
    None,
    Some(7.0),
    None,
    Some(12.0),
    None,
    Some(0.0),
    Some(7.0),
];
/// The melody: step, note, length in steps.
const LEAD: [(usize, f64, usize); 27] = [
    (0, 76.0, 2),
    (3, 81.0, 2),
    (6, 84.0, 2),
    (8, 83.0, 2),
    (10, 81.0, 2),
    (12, 79.0, 2),
    (14, 76.0, 2),
    (16, 77.0, 2),
    (19, 81.0, 2),
    (22, 84.0, 2),
    (24, 86.0, 4),
    (28, 84.0, 2),
    (30, 81.0, 2),
    (32, 79.0, 2),
    (35, 84.0, 2),
    (38, 88.0, 2),
    (40, 86.0, 2),
    (42, 84.0, 2),
    (44, 79.0, 4),
    (48, 83.0, 3),
    (51, 86.0, 3),
    (54, 91.0, 2),
    (56, 88.0, 2),
    (58, 86.0, 2),
    (60, 83.0, 2),
    (62, 86.0, 1),
    (63, 88.0, 1),
];

fn play_step(a: &Audio, step: usize, at: f64, len: f64) {
    let (root, chord) = BARS[step / 16];
    let in_bar = step % 16;
    if in_bar == 0 {
        for m in chord {
            a.tone(&a.music, &Voice { vol: 0.06, ..TRIANGLE }, freq(m), at, len * 6.0, None);
        }
    }
    if let Some(up) = BASS[in_bar] {
        let bass = Voice { wave: OscillatorType::Sawtooth, vol: 0.2, lowpass: Some(700.0) };
        a.tone(&a.music, &bass, freq(root + up), at, len * 0.9, None);
    }
    if let Some((_, m, steps)) = LEAD.iter().find(|(s, _, _)| *s == step) {
        let lead = Voice { wave: OscillatorType::Square, vol: 0.1, lowpass: Some(3200.0) };
        a.tone(&a.music, &lead, freq(*m), at, len * *steps as f64 * 0.95, None);
    }
    if matches!(in_bar, 0 | 6 | 8) {
        a.kick(at, 0.55);
    }
    if matches!(in_bar, 4 | 12) {
        a.snare(at, 0.22);
    } else if step >= 56 {
        a.snare(at, 0.06 + (step - 56) as f32 * 0.02);
    }
    if in_bar.is_multiple_of(2) {
        a.hat(at);
    }
}

/// The march, playing until dropped.
pub struct March {
    timer: Option<IntervalHandle>,
}

impl March {
    pub fn start(a: Rc<Audio>) -> March {
        let at = Cell::new(a.now() + 0.05);
        let step = Cell::new(0usize);
        let timer = set_interval_with_handle(
            move || {
                let now = a.now();
                // Behind after the tab slept: pick up from now rather than rush through the missed notes.
                if at.get() < now - 0.2 {
                    at.set(now + 0.05);
                }
                while at.get() < now + AHEAD {
                    let len = 60.0 / a.tempo.get() / 4.0;
                    play_step(&a, step.get(), at.get(), len);
                    at.set(at.get() + len);
                    step.set((step.get() + 1) % LOOP);
                }
            },
            Duration::from_millis(25),
        )
        .ok();
        March { timer }
    }
}

impl Drop for March {
    fn drop(&mut self) {
        if let Some(t) = self.timer.take() {
            t.clear();
        }
    }
}
