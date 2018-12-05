extern crate wasm_bindgen;
extern crate web_sys;

use wasm_bindgen::prelude::*;
use web_sys::{AudioContext, OscillatorType};

/// Converts a midi note to frequency
///
/// A midi note is an integer, generally in the range of 21 to 108
pub fn midi_to_freq(note: u8) -> f32 {
  27.5 * 2f32.powf((note as f32 - 21.0) / 12.0)
}

#[wasm_bindgen]
pub struct FmOsc {
  ctx: AudioContext,
  primary: web_sys::OscillatorNode, /// The primary oscillator.  This will be the fundamental frequency
  gain: web_sys::GainNode,          /// Overall gain (volume) control
  analyser: web_sys::AnalyserNode,
  fm_gain: web_sys::GainNode,       /// Amount of frequency modulation
  fm_osc: web_sys::OscillatorNode,  /// The oscillator that will modulate the primary oscillator's frequency
  fm_freq_ratio: f32,               /// The ratio between the primary frequency and the fm_osc frequency.
  fm_gain_ratio: f32,
}

impl Drop for FmOsc {
  fn drop(&mut self) {
    let _ = self.ctx.close();
  }
}

#[wasm_bindgen]
impl FmOsc {
  #[wasm_bindgen(constructor)]
  pub fn new() -> Result<FmOsc, JsValue> {
    let ctx = web_sys::AudioContext::new()?;

    // Create our web audio objects.
    let primary = ctx.create_oscillator()?;
    let fm_osc = ctx.create_oscillator()?;
    let gain = ctx.create_gain()?;
    let fm_gain = ctx.create_gain()?;
    let analyser = ctx.create_analyser()?;

    // Some initial settings:
    primary.set_type(OscillatorType::Sine);
    primary.frequency().set_value(440.0); // A4 note
    gain.gain().set_value(0.0);    // starts muted
    fm_gain.gain().set_value(0.0); // no initial frequency modulation
    fm_osc.set_type(OscillatorType::Sine);
    fm_osc.frequency().set_value(0.0);
    analyser.set_fft_size(2048);

    // Connect the nodes up!
    primary.connect_with_audio_node(&gain)?;
    gain.connect_with_audio_node(&ctx.destination())?;
    gain.connect_with_audio_node(&analyser)?;
    fm_osc.connect_with_audio_node(&fm_gain)?;
    fm_gain.connect_with_audio_param(&primary.frequency())?;

    primary.start()?;
    fm_osc.start()?;

    Ok(FmOsc {
        ctx,
        primary,
        gain,
        analyser,
        fm_gain,
        fm_osc,
        fm_freq_ratio: 0.0,
        fm_gain_ratio: 0.0,
    })
  }

  /// Sets the gain for this oscillator, between 0.0 and 1.0.
  #[wasm_bindgen]
  pub fn set_gain(&self, mut gain: f32) {
    if gain > 1.0 {
      gain = 1.0;
    }
    if gain < 0.0 {
      gain = 0.0;
    }
    self.gain.gain().set_value(gain);
  }

  #[wasm_bindgen]
  pub fn set_primary_frequency(&self, freq: f32) {
    self.primary.frequency().set_value(freq);

    // The frequency of the FM oscillator depends on the frequency of the
    // primary oscillator, so we update the frequency of both in this method.
    self.fm_osc.frequency().set_value(self.fm_freq_ratio * freq);
    self.fm_gain.gain().set_value(self.fm_gain_ratio * freq);
  }

  #[wasm_bindgen]
  pub fn set_note(&self, note: u8) {
    let freq = midi_to_freq(note);
    self.set_primary_frequency(freq);
  }

  /// This should be between 0 and 1, though higher values are accepted.
  #[wasm_bindgen]
  pub fn set_fm_amount(&mut self, amt: f32) {
    self.fm_gain_ratio = amt;

    self.fm_gain
        .gain()
        .set_value(self.fm_gain_ratio * self.primary.frequency().value());
  }

  /// This should be between 0 and 1, though higher values are accepted.
  #[wasm_bindgen]
  pub fn set_fm_frequency(&mut self, amt: f32) {
    self.fm_freq_ratio = amt;
    self.fm_osc
        .frequency()
        .set_value(self.fm_freq_ratio * self.primary.frequency().value());
  }

  /* SPECTRUM */

  #[wasm_bindgen]
  pub fn get_buffer_length(&mut self) -> Result<u32, JsValue> {
    let buffer_length = self.analyser.frequency_bin_count();
    Ok(buffer_length)
  }

  /// This should be between 0 and 1, though higher values are accepted.
  #[wasm_bindgen]
  pub fn get_analyser_data(&mut self) -> Result<JsValue, JsValue> {
    let buffer_length = self.analyser.frequency_bin_count();
    // let res = Uint8Array::new(&buffer_length);
    let mut data_array = vec![0u8; buffer_length as usize];
    self.analyser.get_byte_time_domain_data(&mut data_array[..]);

    // Ok(Uint8Array::new(&data_array[..]))
    Ok(JsValue::from_serde(&data_array).unwrap())
  }
}
