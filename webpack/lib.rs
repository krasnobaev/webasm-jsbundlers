extern crate wasm_bindgen;
extern crate web_sys;
extern crate rustfft;
extern crate serde_derive;
extern crate js_sys;

use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
use web_sys::{
  console,
  AudioContext,
  OscillatorType,
  // PeriodicWave,
  WebGlProgram,
  WebGlRenderingContext,
  WebGlShader
};

use rustfft::algorithm::DFT;
use rustfft::FFT;
use rustfft::num_complex::Complex;
use rustfft::num_traits::Zero;

use js_sys::WebAssembly;

extern crate nalgebra_glm as glm;
use glm::Mat4;

static F_VERTEX_SHADER: &'static str = include_str!("shaders/F_v.glsl");
static F_FRAGMENT_SHADER: &'static str = include_str!("shaders/F_f.glsl");
static TRI_VERTEX_SHADER: &'static str = include_str!("shaders/tri_v.glsl");
static TRI_FRAGMENT_SHADER: &'static str = include_str!("shaders/tri_f.glsl");
static CUBE_VERTEX_SHADER: &'static str = include_str!("shaders/cube_v.glsl");
static CUBE_FRAGMENT_SHADER: &'static str = include_str!("shaders/cube_f.glsl");

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

  /*
   * draw spectrum on canvas
   * see https://developer.mozilla.org/en-US/docs/Web/API/Web_Audio_API/Visualizations_with_Web_Audio_API
   */
  pub fn draw_wave(&mut self) {
    let document = web_sys::window().unwrap().document().unwrap();
    let canvas = document.get_element_by_id("waveform").unwrap();
    let canvas: web_sys::HtmlCanvasElement = canvas
        .dyn_into::<web_sys::HtmlCanvasElement>()
        .map_err(|_| ())
        .unwrap();
    let canvas_ctx = canvas
        .get_context("2d")
        .unwrap()
        .unwrap()
        .dyn_into::<web_sys::CanvasRenderingContext2d>()
        .unwrap();

    let buffer_length: usize = self.get_buffer_length() as usize;
    let data: Vec<u8> = self.get_analyser_data_time_domain();

    let width: f64 = canvas.width().into();
    let height: f64 = canvas.height().into();
    let fill_style: JsValue = "rgb(200, 200, 200)".into();
    let stroke_style: JsValue = "rgb(0, 0, 0)".into();

    // var drawVisual = requestAnimationFrame(draw);
    canvas_ctx.set_fill_style(&fill_style);
    canvas_ctx.fill_rect(0.0f64, 0.0f64, width, height);
    canvas_ctx.set_line_width(2.0f64);
    canvas_ctx.set_stroke_style(&stroke_style);
    canvas_ctx.begin_path(); // OK

    let slice_width = width * 1.0f64 / (buffer_length as f64);
    let mut x: f64 = 0.0f64;

    for i in 0..buffer_length {

      let v = (data[i] as f64) / 128.0f64;
      let y = v * height / 2.0f64;

      if i == 0 {
        canvas_ctx.move_to(x, y);
      } else {
        canvas_ctx.line_to(x, y);
      }

      x += slice_width;
    }

    canvas_ctx.stroke();
  }

  pub fn draw_bars(&mut self) {
    let document = web_sys::window().unwrap().document().unwrap();
    let canvas = document.get_element_by_id("spectrum").unwrap();
    let canvas: web_sys::HtmlCanvasElement = canvas
        .dyn_into::<web_sys::HtmlCanvasElement>()
        .map_err(|_| ())
        .unwrap();
    let canvas_ctx = canvas
        .get_context("2d")
        .unwrap()
        .unwrap()
        .dyn_into::<web_sys::CanvasRenderingContext2d>()
        .unwrap();

    let buffer_length: usize = self.get_buffer_length() as usize;
    let data: Vec<u8> = self.get_analyser_data_frequency();

    let width: f64 = canvas.width().into();
    let height: f64 = canvas.height().into();
    let back_fill_style: JsValue = "rgb(200, 200, 200)".into();

    // drawVisual = requestAnimationFrame(draw);
    canvas_ctx.set_fill_style(&back_fill_style);
    canvas_ctx.fill_rect(0.0f64, 0.0f64, width, height);

    let bar_width: f64 = width / (buffer_length as f64);
    let mut x: f64 = 0.;

    // let min: JsValue = JsValue::from_serde(data.iter().min().unwrap()).unwrap().into();
    // let max: JsValue = JsValue::from_serde(data.iter().max().unwrap()).unwrap().into();
    // console::log_2(&"min".into(), &min);
    // console::log_2(&"max".into(), &max);

    for i in 0..buffer_length {
      let bar_height: f64 = (data[i] as f64) / 255.;

      let fill_style_str = &format!("rgb({},50,50)", (bar_height * 130.) + 100.);
      let fill_style: JsValue = fill_style_str.into();
      canvas_ctx.set_fill_style(&fill_style);
      canvas_ctx.fill_rect(x, height - (bar_height * (height - 10.)), bar_width, bar_height * height);

      x += bar_width + 1.;
    }

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
  // pub fn get_buffer_length(&mut self) -> Result<u32, JsValue> {
  pub fn get_buffer_length(&mut self) -> u32 {
    let buffer_length = self.analyser.frequency_bin_count();
    // Ok(buffer_length)
    buffer_length
  }

  /// This should be between 0 and 1, though higher values are accepted.
  #[wasm_bindgen]
  // pub fn get_analyser_data_time_domain(&mut self) -> Result<JsValue, JsValue> {
  pub fn get_analyser_data_time_domain(&mut self) -> Vec<u8> {
    let buffer_length = self.analyser.frequency_bin_count();
    // let res = Uint8Array::new(&buffer_length);
    let mut data_array = vec![0u8; buffer_length as usize];
    self.analyser.get_byte_time_domain_data(&mut data_array[..]);

    // Ok(Uint8Array::new(&data_array[..]))
    // Ok(JsValue::from_serde(&data_array).unwrap())
    data_array
  }

  #[wasm_bindgen]
  pub fn get_analyser_data_frequency(&mut self) -> Vec<u8> {
    let buffer_length = self.analyser.frequency_bin_count();
    let mut data_array = vec![0u8; buffer_length as usize];
    self.analyser.get_byte_frequency_data(&mut data_array[..]);

    data_array
  }
}

/* Web GL */

#[wasm_bindgen]
pub fn drawwebgl() -> Result<(), JsValue> {
    let document = web_sys::window().unwrap().document().unwrap();
    let canvas = document.get_element_by_id("canvas").unwrap();
    let canvas: web_sys::HtmlCanvasElement = canvas.dyn_into::<web_sys::HtmlCanvasElement>()?;

    let context = canvas
        .get_context("webgl")?
        .unwrap()
        .dyn_into::<WebGlRenderingContext>()?;

    // draw_triangle(&context)?;
    // draw_f(&context,
    //   canvas.width() as f32, canvas.height() as f32,
    //   20.0, 80.0
    // )?;
    draw_cube(&context,
      canvas.width() as f32, canvas.height() as f32
    )?;

    Ok(())
}

pub fn draw_triangle (
  context: &WebGlRenderingContext
) -> Result<(), JsValue> {
    let vert_shader = compile_shader(
        &context,
        WebGlRenderingContext::VERTEX_SHADER,
        TRI_VERTEX_SHADER,
    )?;
    let frag_shader = compile_shader(
        &context,
        WebGlRenderingContext::FRAGMENT_SHADER,
        TRI_FRAGMENT_SHADER,
    )?;

    let program = link_program(&context, [vert_shader, frag_shader].iter())?;
    context.use_program(Some(&program));

    context.clear_color(0.0, 0.0, 0.0, 1.0);
    context.clear(WebGlRenderingContext::COLOR_BUFFER_BIT);

    set_tri_geometry(context)?;

    Ok(())
}

pub fn set_tri_geometry(
  context: &WebGlRenderingContext
) -> Result<(), JsValue> {
  let vertices: [f32; 9] = [-0.7, -0.7, 0.0, 0.7, -0.7, 0.0, 0.0, 0.7, 0.0];
  let memory_buffer = wasm_bindgen::memory()
      .dyn_into::<WebAssembly::Memory>()?
      .buffer();
  let vertices_location = vertices.as_ptr() as u32 / 4;
  let vert_array = js_sys::Float32Array::new(&memory_buffer)
      .subarray(vertices_location, vertices_location + vertices.len() as u32);

  let buffer = context.create_buffer().ok_or("failed to create buffer")?;
  context.bind_buffer(WebGlRenderingContext::ARRAY_BUFFER, Some(&buffer));
  context.buffer_data_with_array_buffer_view(
      WebGlRenderingContext::ARRAY_BUFFER,
      &vert_array,
      WebGlRenderingContext::STATIC_DRAW,
  );
  context.vertex_attrib_pointer_with_i32(0, 3, WebGlRenderingContext::FLOAT, false, 0, 0);
  context.enable_vertex_attrib_array(0);

  context.draw_arrays(
      WebGlRenderingContext::TRIANGLES,
      0,
      (vertices.len() / 3) as i32,
  );

  Ok(())
}

pub fn draw_f (
  context: &WebGlRenderingContext,
  width: f32,
  height: f32,
  translationx: f32,
  translationy: f32,
) -> Result<(), JsValue> {
    let vert_shader = compile_shader(
        &context,
        WebGlRenderingContext::VERTEX_SHADER,
        F_VERTEX_SHADER,
    )?;
    let frag_shader = compile_shader(
        &context,
        WebGlRenderingContext::FRAGMENT_SHADER,
        F_FRAGMENT_SHADER,
    )?;

    let program = link_program(&context, [vert_shader, frag_shader].iter())?;
    context.use_program(Some(&program));

    // look up where the vertex data needs to go
    let position_location: u32 = context.get_attrib_location(&program, "a_position") as u32;
    let bar_scale: u32 = context.get_attrib_location(&program, "a_scale") as u32;

    // set the resolution
    let ures = context.get_uniform_location(&program, "u_resolution");
    context.uniform2f(ures.as_ref(), width, height);

    // set the color
    let uloc = context.get_uniform_location(&program, "u_color");
    let mut color: [f32; 4] = [0.1, 0.2, 0.3, 0.4];
    context.uniform4fv_with_f32_array(uloc.as_ref(), &mut color);

    // Set the translation
    let utrans = context.get_uniform_location(&program, "u_translation");
    let mut translation: [f32; 2] = [translationx, translationy];
    context.uniform2fv_with_f32_array(utrans.as_ref(), &mut translation);

    // position buffer
    let buffer = context.create_buffer().ok_or("failed to create buffer")?;
    context.bind_buffer(WebGlRenderingContext::ARRAY_BUFFER, Some(&buffer));

    // cleanup
    context.clear_color(0.0, 0.0, 0.0, 1.0);
    context.clear(WebGlRenderingContext::COLOR_BUFFER_BIT);

    init_f_buffers(context)?;

    // enable vertices
    context.enable_vertex_attrib_array(position_location);
    context.vertex_attrib_pointer_with_i32(
      position_location, 2, WebGlRenderingContext::UNSIGNED_BYTE, false, 0, 0
    );

    // enable colors
    context.enable_vertex_attrib_array(bar_scale);
    context.vertex_attrib_pointer_with_i32(
      bar_scale, 1, WebGlRenderingContext::UNSIGNED_BYTE, true, 0, 0
    );

    // enable indices
    // context.enable_vertex_attrib_array(position_location???);
    // context.vertex_attrib_pointer_with_i32(
    //   position_location???, 2, WebGlRenderingContext::UNSIGNED_BYTE, false, 0, 0
    // );

    context.draw_arrays(
        WebGlRenderingContext::TRIANGLES,
        0,
        18, // (vertices.len() / 2) as i32,
    );

    Ok(())
}

pub fn init_f_buffers(
  context: &WebGlRenderingContext
) -> Result<(), JsValue> {
  // set vertices
  let vertices: [u8; 36] = [
    // first bar column - 6
    0, 0,
    30, 0,
    0, 150,
    0, 150,
    30, 0,
    30, 150,

    // second bar column - 6
    40, 0,
    70, 0,
    40, 150,
    40, 150,
    70, 0,
    70, 150,

    // third bar column - 6
    80, 0,
    110, 0,
    80, 150,
    80, 150,
    110, 0,
    110, 150,
  ];

  let memory_buffer = wasm_bindgen::memory()
      .dyn_into::<WebAssembly::Memory>()?
      .buffer();
  let vertices_location = vertices.as_ptr() as u32 / 4;
  let vert_array = js_sys::Float32Array::new(&memory_buffer)
      .subarray(vertices_location, vertices_location + 36u32);

  let buffer = context.create_buffer().ok_or("failed to create buffer")?;
  context.bind_buffer(WebGlRenderingContext::ARRAY_BUFFER, Some(&buffer));
  context.buffer_data_with_array_buffer_view(
      WebGlRenderingContext::ARRAY_BUFFER,
      &vert_array,
      WebGlRenderingContext::STATIC_DRAW,
  );

  // set colors
  // let colors: [f32; 12] = [
  //   0.3, 0.3, 0.3, 0.3,
  //   0.6, 0.6, 0.6, 0.6,
  //   0.9, 0.9, 0.9, 0.9,
  // ];

  // let colors_mem_buffer = wasm_bindgen::memory()
  //     .dyn_into::<WebAssembly::Memory>()?
  //     .buffer();
  // let colors_location = colors.as_ptr() as u32 / 2;
  // let colors_array = js_sys::Float32Array::new(&colors_mem_buffer)
  //     .subarray(colors_location, colors_location + colors.len() as u32);

  // let colors_buffer = context.create_buffer().ok_or("failed to create buffer")?;
  // context.bind_buffer(WebGlRenderingContext::ARRAY_BUFFER, Some(&colors_buffer));
  // context.buffer_data_with_array_buffer_view(
  //     WebGlRenderingContext::ARRAY_BUFFER,
  //     &colors_array,
  //     WebGlRenderingContext::STATIC_DRAW,
  // );

  // set indices
  // let indices: [u8; 18] = [
  //   // first bar column - 6
  //   1, 2, 3, 4, 5, 6,
  //   // second bar column - 6
  //   7, 8, 9, 10, 11, 12,
  //   // third bar column - 6
  //   13, 14, 15, 16, 17, 18,
  // ];

  // let memory_buffer = wasm_bindgen::memory()
  //     .dyn_into::<WebAssembly::Memory>()?
  //     .buffer();
  // let indices_location = indices.as_ptr() as u32 / 4;
  // let indc_array = js_sys::Uint8Array::new(&memory_buffer)
  //     .subarray(indices_location, indices_location + indices.len() as u32);

  // let buffer = context.create_buffer().ok_or("failed to create buffer")?;
  // context.bind_buffer(WebGlRenderingContext::ELEMENT_ARRAY_BUFFER, Some(&buffer));
  // context.buffer_data_with_array_buffer_view(
  //     WebGlRenderingContext::ELEMENT_ARRAY_BUFFER,
  //     &indc_array,
  //     WebGlRenderingContext::STATIC_DRAW,
  // );

  // set indices
  // context.vertex_attrib_pointer_with_i32(
  //   position_location, 2, WebGlRenderingContext::UNSIGNED_BYTE, false, 0, 0
  // );
  // context.enable_vertex_attrib_array(position_location);

  Ok(())
}

pub fn draw_cube (
  context: &WebGlRenderingContext,
  width: f32,
  height: f32,
) -> Result<(), JsValue> {
  let cube_rotation = 0.3;

  let vert_shader = compile_shader(
      &context,
      WebGlRenderingContext::VERTEX_SHADER,
      CUBE_VERTEX_SHADER,
  )?;
  let frag_shader = compile_shader(
      &context,
      WebGlRenderingContext::FRAGMENT_SHADER,
      CUBE_FRAGMENT_SHADER,
  )?;

  // Tell WebGL to use our program when drawing
  let program = link_program(&context, [vert_shader, frag_shader].iter())?;

  //
  // fill vertices buffer
  //

  let vertices: [f32; 72] = [
    // Front face
    -1.0, -1.0,  1.0,
    1.0, -1.0,  1.0,
    1.0,  1.0,  1.0,
    -1.0,  1.0,  1.0,

    // Back face
    -1.0, -1.0, -1.0,
    -1.0,  1.0, -1.0,
    1.0,  1.0, -1.0,
    1.0, -1.0, -1.0,

    // Top face
    -1.0,  1.0, -1.0,
    -1.0,  1.0,  1.0,
    1.0,  1.0,  1.0,
    1.0,  1.0, -1.0,

    // Bottom face
    -1.0, -1.0, -1.0,
    1.0, -1.0, -1.0,
    1.0, -1.0,  1.0,
    -1.0, -1.0,  1.0,

    // Right face
    1.0, -1.0, -1.0,
    1.0,  1.0, -1.0,
    1.0,  1.0,  1.0,
    1.0, -1.0,  1.0,

    // Left face
    -1.0, -1.0, -1.0,
    -1.0, -1.0,  1.0,
    -1.0,  1.0,  1.0,
    -1.0,  1.0, -1.0,
  ];

  let position_buffer = context.create_buffer().ok_or("failed to create buffer")?;
  context.bind_buffer(WebGlRenderingContext::ARRAY_BUFFER, Some(&position_buffer));

  let position_memory_buffer = wasm_bindgen::memory()
      .dyn_into::<WebAssembly::Memory>()?
      .buffer();
  let vertices_location = vertices.as_ptr() as u32 / 4;
  let vert_array = js_sys::Float32Array::new(&position_memory_buffer)
      .subarray(vertices_location, vertices_location + vertices.len() as u32);

  context.buffer_data_with_array_buffer_view(
      WebGlRenderingContext::ARRAY_BUFFER,
      &vert_array,
      WebGlRenderingContext::STATIC_DRAW,
  );

  //
  // fill colors buffer
  //

  let colors: [f32; 24] = [
    1.0,  1.0,  1.0,  1.0,    // Front face: white
    1.0,  0.0,  0.0,  1.0,    // Back face: red
    0.0,  1.0,  0.0,  1.0,    // Top face: green
    0.0,  0.0,  1.0,  1.0,    // Bottom face: blue
    1.0,  1.0,  0.0,  1.0,    // Right face: yellow
    1.0,  0.0,  1.0,  1.0,    // Left face: purple
  ];

  let color_buffer = context.create_buffer().ok_or("failed to create buffer")?;
  context.bind_buffer(WebGlRenderingContext::ARRAY_BUFFER, Some(&color_buffer));

  let colors_memory_buffer = wasm_bindgen::memory()
      .dyn_into::<WebAssembly::Memory>()?
      .buffer();
  let colors_location = vertices.as_ptr() as u32 / 4;
  let colors_array = js_sys::Float32Array::new(&colors_memory_buffer)
      .subarray(colors_location, colors_location + colors.len() as u32);

  context.buffer_data_with_array_buffer_view(
      WebGlRenderingContext::ARRAY_BUFFER,
      &colors_array,
      WebGlRenderingContext::STATIC_DRAW,
  );

  //
  // fill indices buffer
  //

  let indices: [i8; 36] = [
    0,  1,  2,      0,  2,  3,    // front
    4,  5,  6,      4,  6,  7,    // back
    8,  9,  10,     8,  10, 11,   // top
    12, 13, 14,     12, 14, 15,   // bottom
    16, 17, 18,     16, 18, 19,   // right
    20, 21, 22,     20, 22, 23,   // left
  ];

  let ind_buffer = context.create_buffer().ok_or("failed to create buffer")?;
  context.bind_buffer(WebGlRenderingContext::ELEMENT_ARRAY_BUFFER, Some(&ind_buffer));

  let inds_memory_buffer = wasm_bindgen::memory()
      .dyn_into::<WebAssembly::Memory>()?
      .buffer();
  let inds_location = indices.as_ptr() as u32 / 4;
  let inds_array = js_sys::Uint16Array::new(&inds_memory_buffer)
      .subarray(inds_location, inds_location + indices.len() as u32);

  context.buffer_data_with_array_buffer_view(
      WebGlRenderingContext::ELEMENT_ARRAY_BUFFER,
      &inds_array,
      WebGlRenderingContext::STATIC_DRAW,
  );

  //
  // draw the scene
  //

  // Clear to black, fully opaque
  context.clear_color(0.0, 0.0, 0.0, 1.0);
  // Clear everything
  context.clear_depth(1.0);
  // Enable depth testing
  context.enable(WebGlRenderingContext::DEPTH_TEST);
  // Near things obscure far things
  context.depth_func(WebGlRenderingContext::LEQUAL);
  // Clear the canvas before we start drawing on it.
  context.clear(WebGlRenderingContext::COLOR_BUFFER_BIT);

  //
  // calc perspective and rotation
  //

  // Create a perspective matrix, a special matrix that is
  // used to simulate the distortion of perspective in a camera.
  // Our field of view is 45 degrees, with a width/height
  // ratio that matches the display size of the canvas
  // and we only want to see objects between 0.1 units
  // and 100 units away from the camera.

  let field_of_view = 45.0 * std::f32::consts::PI / 180.0;   // in radians
  let aspect = width / height;
  let z_near = 0.1;
  let z_far = 100.0;
  let projection_matrix_data = glm::perspective(field_of_view, aspect, z_near, z_far);

  let mut model_view_matrix_data = Mat4::identity();
  let translation = glm::vec3(-0.0, 0.0, -6.0);
  model_view_matrix_data = glm::translate(&model_view_matrix_data, &translation);
  model_view_matrix_data = glm::rotate_z(&model_view_matrix_data, cube_rotation);
  model_view_matrix_data = glm::rotate_x(&model_view_matrix_data, cube_rotation * 0.7);

  //
  // aVertexPosition
  //

  // Tell WebGL how to pull out the positions from the position
  // buffer into the vertexPosition attribute
  let num_components = 3;
  let normalize = false;
  let stride = 0;
  let offset = 0;
  let vertex_position: u32 = context.get_attrib_location(&program, "aVertexPosition") as u32;
  context.bind_buffer(WebGlRenderingContext::ARRAY_BUFFER, Some(&position_buffer));
  context.vertex_attrib_pointer_with_i32(
      vertex_position,
      num_components,
      WebGlRenderingContext::FLOAT,
      normalize,
      stride,
      offset);
  context.enable_vertex_attrib_array(vertex_position);

  //
  // aVertexColor
  //

  // Tell WebGL how to pull out the colors from the color buffer
  // into the vertexColor attribute.
  let num_components = 4;
  let normalize = false;
  let stride = 0;
  let offset = 0;
  let vertex_color: u32 = context.get_attrib_location(&program, "aVertexColor") as u32;
  context.bind_buffer(WebGlRenderingContext::ARRAY_BUFFER, Some(&color_buffer));
  context.vertex_attrib_pointer_with_i32(
      vertex_color,
      num_components,
      WebGlRenderingContext::FLOAT,
      normalize,
      stride,
      offset);
  context.enable_vertex_attrib_array(vertex_color);

  // Tell WebGL which indices to use to index the vertices
  context.bind_buffer(WebGlRenderingContext::ELEMENT_ARRAY_BUFFER, Some(&ind_buffer));

  // Tell WebGL to use our program when drawing
  context.use_program(Some(&program));

  //
  // Set the shader uniforms
  //

  // uProjectionMatrix
  let projection_matrix = context.get_uniform_location(&program, "uProjectionMatrix");
  // let mut projection_matrix_data_static: [f32; 16] = [
  //   1.81,  0.0,    0.0,   0.0,
  //    0.0, 2.41,    0.0,   0.0,
  //    0.0,  0.0,  -1.00,  -1.0,
  //    0.0,  0.0, -0.200,   0.0,
  // ];
  context.uniform_matrix4fv_with_f32_array(
      projection_matrix.as_ref(),
      false,
      &mut flatten(projection_matrix_data),
      // &mut projection_matrix_data_static,
  );

  // uModelViewMatrix
  let model_view_matrix = context.get_uniform_location(&program, "uModelViewMatrix");
  // let mut model_view_matrix_data_static: [f32; 16] = [
  //   -0.36,  0.42, -0.82, 0.00,
  //   -0.75, -0.65,  0.00, 0.00,
  //   -0.53,  0.62,  0.56, 0.00,
  //    0.00,  0.00, -6.00, 1.00,
  // ];
  context.uniform_matrix4fv_with_f32_array(
      model_view_matrix.as_ref(),
      false,
      &mut flatten(model_view_matrix_data),
      // &mut model_view_matrix_data_static,
  );

  //
  // Draw
  //

  context.draw_elements_with_i32(
      WebGlRenderingContext::TRIANGLES,
      36, // vertexCount
      WebGlRenderingContext::UNSIGNED_SHORT,
      0 // offset
  );

  Ok(())
}

pub fn flatten (data: Mat4) -> Vec<f32> {
  data.iter()
      // .flat_map(|array| array.as_array().iter())
      .cloned()
      .collect()
}

pub fn compile_shader(
    context: &WebGlRenderingContext,
    shader_type: u32,
    source: &str,
) -> Result<WebGlShader, String> {
  let shader = context
      .create_shader(shader_type)
      .ok_or_else(|| String::from("Unable to create shader object"))?;
  context.shader_source(&shader, source);
  context.compile_shader(&shader);

  if context
      .get_shader_parameter(&shader, WebGlRenderingContext::COMPILE_STATUS)
      .as_bool()
      .unwrap_or(false)
  {
    Ok(shader)
  } else {
    Err(context
        .get_shader_info_log(&shader)
        .unwrap_or_else(|| "Unknown error creating shader".into()))
  }
}

pub fn link_program<'a, T: IntoIterator<Item = &'a WebGlShader>>(
    context: &WebGlRenderingContext,
    shaders: T,
) -> Result<WebGlProgram, String> {
  let program = context
      .create_program()
      .ok_or_else(|| String::from("Unable to create shader object"))?;
  for shader in shaders {
    context.attach_shader(&program, shader)
  }
  context.link_program(&program);

  if context
      .get_program_parameter(&program, WebGlRenderingContext::LINK_STATUS)
      .as_bool()
      .unwrap_or(false)
  {
    Ok(program)
  } else {
    Err(context
        .get_program_info_log(&program)
        .unwrap_or_else(|| "Unknown error creating program object".into()))
  }
}
