extern crate wasm_bindgen;
extern crate web_sys;
extern crate rustfft;
extern crate serde_derive;

use wasm_bindgen::prelude::*;
use web_sys::{AudioContext, OscillatorType, PeriodicWave};

use rustfft::algorithm::DFT;
use rustfft::FFT;
use rustfft::num_complex::Complex;
use rustfft::num_traits::Zero;

/*
 * DFT
 */

pub fn getdft(data: &JsValue) -> Result<Vec<Vec<f32>>, JsValue> {
  let data2: Vec<f32> = data.into_serde().unwrap();
  let buflen: usize = data2.len();

  let mut input:  Vec<Complex<f32>> = data2.iter().map(|&x| Complex::new(x, 0.0f32)).collect();
  let mut output: Vec<Complex<f32>> = vec![Zero::zero(); buflen];
  let dft = DFT::new(buflen, false);
  dft.process(&mut input, &mut output);

  let real: Vec<_> = output.iter().map(|&x| x.re).collect();
  let imag: Vec<_> = output.iter().map(|&x| x.im).collect();

  // Ok(JsValue::from_serde(&vec![&real, &imag]).unwrap())
  Ok(vec![real, imag].to_vec())
}

/// Converts a midi note to frequency
///
/// A midi note is an integer, generally in the range of 21 to 108
pub fn midi_to_freq(note: u8) -> f32 {
  27.5 * 2f32.powf((note as f32 - 21.0) / 12.0)
}

/*
 * Synth
 */

#[wasm_bindgen]
pub struct FmOsc {
  ctx: AudioContext,
  osc1: web_sys::OscillatorNode,  // this will be the fundamental frequency
  osc1_wave_type: u8,
  osc1_gain: web_sys::GainNode,

  osc2: web_sys::OscillatorNode,  // this will modulate the osc1 oscillator's frequency
  osc2_wave_type: u8,
  osc2_gain: web_sys::GainNode,   // Amount of frequency modulation
  osc2_gain_value: f32,

  fmfreq_1_to_2: f32,             // The ratio between the osc1 frequency and the osc2 frequency.

  analyser: web_sys::AnalyserNode,
  ms_gain: web_sys::GainNode,     // Overall gain (volume) control
}

impl Drop for FmOsc {
  fn drop(&mut self) {
    let _ = self.ctx.close();
  }
}

#[wasm_bindgen]
impl FmOsc {
  #[wasm_bindgen(constructor)]
  pub fn new(data: &JsValue) -> Result<FmOsc, JsValue> {
    let ctx = web_sys::AudioContext::new()?;

    // Create our web audio objects.
    let osc1 = ctx.create_oscillator()?;
    let osc2 = ctx.create_oscillator()?;
    let osc1_gain = ctx.create_gain()?;
    let osc2_gain = ctx.create_gain()?;

    let analyser = ctx.create_analyser()?;
    let ms_gain = ctx.create_gain()?;

    // let pdata: Vec<Vec<f32>> = getdft(data)?;
    // let mut real: Vec<f32> = pdata[0][..].to_vec();
    // let mut imag: Vec<f32> = pdata[1][..].to_vec();
    // let customwave = ctx.create_periodic_wave(&mut real, &mut imag)?;

    // Some initial settings:
    osc1.set_type(OscillatorType::Sine);
    osc1.frequency().set_value(440.0); // A4 note
    osc1_gain.gain().set_value(0.0);    // starts muted
    osc2_gain.gain().set_value(0.0); // no initial frequency modulation
    osc2.set_type(OscillatorType::Sine);
    osc2.frequency().set_value(0.0);
    analyser.set_fft_size(2048);
    ms_gain.gain().set_value(0.0); // starts muted

    // Connect the nodes up!
    osc1.connect_with_audio_node(&osc1_gain)?;
    osc2.connect_with_audio_node(&osc2_gain)?;
    osc2_gain.connect_with_audio_param(&osc1.frequency())?;
    osc1_gain.connect_with_audio_node(&ms_gain)?;
    osc1_gain.connect_with_audio_node(&analyser)?;

    ms_gain.connect_with_audio_node(&ctx.destination())?;

    osc1.start()?;
    osc2.start()?;

    Ok(FmOsc {
        ctx,
        osc1,
        osc1_wave_type: 1,
        osc1_gain,

        osc2,
        osc2_wave_type: 1,
        osc2_gain,
        osc2_gain_value: 0.0,

        fmfreq_1_to_2: 0.0,

        analyser,
        ms_gain,
    })
  }

  #[wasm_bindgen]
  pub fn set_osc1_wave_type(&mut self, wave: &str) {
    self.osc1_wave_type = match wave {
      // "cst" => 0,
      "sin" => 1,
      "tri" => 2,
      "sqr" => 3,
      "saw" => 4,
      _ => 255,
    };

    match self.osc1_wave_type {
      // 0 => self.osc1.set_periodic_wave(&customwave);,
      1 => self.osc1.set_type(OscillatorType::Sine),
      2 => self.osc1.set_type(OscillatorType::Triangle),
      3 => self.osc1.set_type(OscillatorType::Square),
      4 => self.osc1.set_type(OscillatorType::Sawtooth),
      _ => ()
    };

  }

  #[wasm_bindgen]
  pub fn set_osc2_wave_type(&mut self, wave: &str) {
    self.osc2_wave_type = match wave {
      // "cst" => 0,
      "sin" => 1,
      "tri" => 2,
      "sqr" => 3,
      "saw" => 4,
      _ => 255,
    };

    match self.osc2_wave_type {
      // 0 => self.osc1.set_periodic_wave(&customwave);,
      1 => self.osc2.set_type(OscillatorType::Sine),
      2 => self.osc2.set_type(OscillatorType::Triangle),
      3 => self.osc2.set_type(OscillatorType::Square),
      4 => self.osc2.set_type(OscillatorType::Sawtooth),
      _ => ()
    };

  }

  /// Sets the gain for this oscillator, between 0.0 and 1.0.
  #[wasm_bindgen]
  pub fn set_osc1_gain(&self, mut gain: f32) {
    if gain > 1.0 {
      gain = 1.0;
    }
    if gain < 0.0 {
      gain = 0.0;
    }
    self.osc1_gain.gain().set_value(gain);
  }

  #[wasm_bindgen]
  pub fn set_osc1_frequency(&self, freq: f32) {
    self.osc1.frequency().set_value(freq);

    // The frequency of the FM oscillator depends on the frequency of the
    // osc1 oscillator, so we update the frequency of both in this method.
    self.osc2.frequency().set_value(self.fmfreq_1_to_2 * freq);
    self.osc2_gain.gain().set_value(self.osc2_gain_value * freq);
  }

  #[wasm_bindgen]
  pub fn set_note(&self, note: u8) {
    let freq = midi_to_freq(note);
    self.set_osc1_frequency(freq);
  }

  /// This should be between 0 and 1, though higher values are accepted.
  #[wasm_bindgen]
  pub fn set_fm_amount(&mut self, amt: f32) {
    self.osc2_gain_value = amt;

    self.osc2_gain
        .gain()
        .set_value(self.osc2_gain_value * self.osc1.frequency().value());
  }

  /// This should be between 0 and 1, though higher values are accepted.
  #[wasm_bindgen]
  pub fn set_fm_frequency(&mut self, amt: f32) {
    self.fmfreq_1_to_2 = amt;
    self.osc2
        .frequency()
        .set_value(self.fmfreq_1_to_2 * self.osc1.frequency().value());
  }

  #[wasm_bindgen]
  pub fn set_ms_gain(&mut self, mut gain: f32) {
    if gain > 1.0 {
      gain = 1.0;
    }
    if gain < 0.0 {
      gain = 0.0;
    }
    self.ms_gain.gain().set_value(gain);
  }

  /*
   * SPECTRUM
   *
   * see https://developer.mozilla.org/en-US/docs/Web/API/Web_Audio_API/Visualizations_with_Web_Audio_API
   */

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
