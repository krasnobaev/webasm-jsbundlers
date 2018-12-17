extern crate wasm_bindgen;
extern crate web_sys;
extern crate rustfft;
extern crate serde_derive;

use wasm_bindgen::prelude::*;
use web_sys::{
  console,
  AudioContext,
  OscillatorType,
  // PeriodicWave,
};

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
  base_freq: f32,

  osc1: web_sys::OscillatorNode,  // this will be the fundamental frequency
  osc1_wave_type: u8,
  osc1_gain: web_sys::GainNode,
  osc1_gain_value: f32,
  osc1_bypass: web_sys::GainNode,
  osc1_bypass_value: f32,

  osc2: web_sys::OscillatorNode,  // this will modulate the osc1 oscillator's frequency
  osc2_wave_type: u8,
  osc2_gain: web_sys::GainNode,
  osc2_gain_value: f32,
  osc2_bypass: web_sys::GainNode,
  osc2_bypass_value: f32,

  // fm matrix
  osc1t1gain: web_sys::GainNode,
  osc2t1gain: web_sys::GainNode,
  osc1t2gain: web_sys::GainNode,
  osc2t2gain: web_sys::GainNode,

  // fm ratio between osc's
  fmfreq_1t2: f32,
  fmfreq_1t1: f32,
  fmfreq_2t1: f32,
  fmfreq_2t2: f32,

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
    let osc1_bypass = ctx.create_gain()?;
    let osc2_bypass = ctx.create_gain()?;

    // fm matrix
    let osc1t1gain = ctx.create_gain()?;
    let osc2t1gain = ctx.create_gain()?;
    let osc1t2gain = ctx.create_gain()?;
    let osc2t2gain = ctx.create_gain()?;

    let analyser = ctx.create_analyser()?;
    let ms_gain = ctx.create_gain()?;

    // let pdata: Vec<Vec<f32>> = getdft(data)?;
    // let mut real: Vec<f32> = pdata[0][..].to_vec();
    // let mut imag: Vec<f32> = pdata[1][..].to_vec();
    // let customwave = ctx.create_periodic_wave(&mut real, &mut imag)?;

    // Some initial settings:
    osc1.set_type(OscillatorType::Sine);
    osc1.frequency().set_value(0.0);
    osc1_gain.gain().set_value(0.0);
    osc1_bypass.gain().set_value(0.0);

    osc2.set_type(OscillatorType::Sine);
    osc2.frequency().set_value(0.0);
    osc2_gain.gain().set_value(0.0);
    osc2_bypass.gain().set_value(0.0);

    // fm matrix
    osc1t1gain.gain().set_value(0.0);
    osc2t1gain.gain().set_value(0.0);
    osc1t2gain.gain().set_value(0.0);
    osc2t2gain.gain().set_value(0.0);

    analyser.set_fft_size(2048);
    ms_gain.gain().set_value(0.0); // starts muted

    // OSC -> gain
    osc1.connect_with_audio_node(&osc1_gain)?;
    osc2.connect_with_audio_node(&osc2_gain)?;
    osc1.connect_with_audio_node(&osc1_bypass)?;
    osc2.connect_with_audio_node(&osc2_bypass)?;

    // fm matrix
    osc1_gain.connect_with_audio_node(&osc1t1gain)?;
    osc1_gain.connect_with_audio_node(&osc1t2gain)?;
    osc2_gain.connect_with_audio_node(&osc2t1gain)?;
    osc2_gain.connect_with_audio_node(&osc2t2gain)?;
    osc1t1gain.connect_with_audio_param(&osc1.frequency())?;
    osc2t1gain.connect_with_audio_param(&osc1.frequency())?;
    osc1t2gain.connect_with_audio_param(&osc2.frequency())?;
    osc2t2gain.connect_with_audio_param(&osc2.frequency())?;

    // mix to main bus
    osc1_bypass.connect_with_audio_node(&ms_gain)?;
    osc2_bypass.connect_with_audio_node(&ms_gain)?;

    ms_gain.connect_with_audio_node(&analyser)?;
    ms_gain.connect_with_audio_node(&ctx.destination())?;

    osc1.start()?;
    osc2.start()?;

    Ok(FmOsc {
        ctx,
        base_freq: 0.0,

        osc1,
        osc1_wave_type: 1,
        osc1_gain,
        osc1_gain_value: 0.0,
        osc1_bypass,
        osc1_bypass_value: 0.0,

        osc2,
        osc2_wave_type: 1,
        osc2_gain,
        osc2_gain_value: 0.0,
        osc2_bypass,
        osc2_bypass_value: 0.0,

        osc1t1gain,
        osc2t1gain,
        osc1t2gain,
        osc2t2gain,

        fmfreq_1t2: 0.0,
        fmfreq_1t1: 0.0,
        fmfreq_2t1: 0.0,
        fmfreq_2t2: 0.0,

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
  pub fn set_osc1_gain(&mut self, mut gain: f32) {
    if gain > 1.0 {
      gain = 1.0;
    }
    if gain < 0.0 {
      gain = 0.0;
    }

    self.osc1_gain_value = gain;
    self.osc1_gain.gain().set_value(self.osc1_gain_value);
  }

  #[wasm_bindgen]
  pub fn set_osc2_gain(&mut self, mut gain: f32) {
    if gain > 1.0 {
      gain = 1.0;
    }
    if gain < 0.0 {
      gain = 0.0;
    }

    self.osc2_gain_value = gain;
    self.osc2_gain.gain().set_value(self.osc1_gain_value);
  }

  #[wasm_bindgen]
  pub fn set_osc1_bypass(&mut self, mut gain: f32) {
    if gain > 1.0 {
      gain = 1.0;
    }
    if gain < 0.0 {
      gain = 0.0;
    }

    self.osc1_bypass_value = gain;
    self.osc1_bypass.gain().set_value(self.osc1_bypass_value);
  }

  #[wasm_bindgen]
  pub fn set_osc2_bypass(&mut self, mut gain: f32) {
    if gain > 1.0 {
      gain = 1.0;
    }
    if gain < 0.0 {
      gain = 0.0;
    }

    self.osc2_bypass_value = gain;
    self.osc2_bypass.gain().set_value(self.osc2_bypass_value);
  }

  #[wasm_bindgen]
  pub fn adjust_fm_matrix(&self) {
    let freq: f32 = self.base_freq;

    self.osc1t1gain.gain().set_value(self.fmfreq_1t1 * freq);
    self.osc1t2gain.gain().set_value(self.fmfreq_1t2 * freq);
    self.osc2t1gain.gain().set_value(self.fmfreq_2t1 * freq);
    self.osc2t2gain.gain().set_value(self.fmfreq_2t2 * freq);

    // self.osc1.frequency().set_value(
    //   self.fmfreq_1t1 * freq + self.fmfreq_2t1 * freq + freq
    // );
    // self.osc2.frequency().set_value(
    //   self.fmfreq_1t2 * freq + self.fmfreq_2t2 * freq + freq
    // );

    let fm1: JsValue = (self.fmfreq_1t1 * freq).into();
    let fm2: JsValue = (self.fmfreq_1t2 * freq).into();
    let fm3: JsValue = (self.fmfreq_2t1 * freq).into();
    let fm4: JsValue = (self.fmfreq_2t2 * freq).into();
    console::log_2(&"osc1t1gain".into(), &fm1);
    console::log_2(&"osc1t2gain".into(), &fm2);
    console::log_2(&"osc2t1gain".into(), &fm3);
    console::log_2(&"osc2t2gain".into(), &fm4);
  }

  #[wasm_bindgen]
  pub fn set_osc_frequency(&self, freq: f32) {
    self.osc1.frequency().set_value(freq);
    self.osc2.frequency().set_value(freq);

    self.adjust_fm_matrix();
  }

  #[wasm_bindgen]
  pub fn set_note(&mut self, note: u8) {
    let freq = midi_to_freq(note);
    self.base_freq = freq;
    self.set_osc_frequency(self.base_freq);
  }

  #[wasm_bindgen]
  pub fn set_fm1to1(&mut self, amt: f32) {
    self.fmfreq_1t1 = amt;
    self.osc1t1gain.gain().set_value(self.fmfreq_1t1);
    self.adjust_fm_matrix();
  }

  #[wasm_bindgen]
  pub fn set_fm1to2(&mut self, amt: f32) {
    self.fmfreq_1t2 = amt;
    self.osc1t2gain.gain().set_value(self.fmfreq_1t2);
    self.adjust_fm_matrix();
  }

  #[wasm_bindgen]
  pub fn set_fm2to1(&mut self, amt: f32) {
    self.fmfreq_2t1 = amt;
    self.osc1t1gain.gain().set_value(self.fmfreq_2t1);
    self.adjust_fm_matrix();
  }

  #[wasm_bindgen]
  pub fn set_fm2to2(&mut self, amt: f32) {
    self.fmfreq_2t2 = amt;
    self.osc2t1gain.gain().set_value(self.fmfreq_2t2);
    self.adjust_fm_matrix();
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
