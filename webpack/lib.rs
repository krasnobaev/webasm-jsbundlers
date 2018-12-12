extern crate wasm_bindgen;
extern crate web_sys;
extern crate rustfft;
extern crate serde_derive;
// extern crate wasm_bindgen_futures;
// extern crate futures;

use wasm_bindgen::prelude::*;
use web_sys::{
  AudioContext,
  // Navigator,
  OscillatorType,
  // PeriodicWave
};

use rustfft::algorithm::DFT;
use rustfft::FFT;
use rustfft::num_complex::Complex;
use rustfft::num_traits::Zero;

/* Provider */
#[wasm_bindgen]
pub struct Provider {
  fmosc: FmOsc,
  midiinput: MIDIInput,
}

#[wasm_bindgen]
impl Provider {
  #[wasm_bindgen(constructor)]
  pub fn new(data: &JsValue) -> Result<Provider, JsValue> {
    let fmosc = FmOsc::new(data);
    let midiinput = MIDIInput::new();

    Ok(FmOsc {
        fmosc,
        midiinput
    })
  }
}

// MIDI
// use futures::{Async, Future, Poll};
// use wasm_bindgen_futures::{JsFuture, future_to_promise};

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

/* NextTick */

/// A future that becomes ready after a tick of the micro task queue.
// pub struct NextTick {
//   inner: JsFuture,
// }
// impl NextTick {
//   /// Construct a new `NextTick` future.
//   pub fn new() -> NextTick {
//     // Create a resolved promise that will run its callbacks on the next
//     // tick of the micro task queue.
//     let promise = js_sys::Promise::resolve(&JsValue::NULL);
//     // Convert the promise into a `JsFuture`.
//     let inner = JsFuture::from(promise);
//     NextTick { inner }
//   }
// }
// impl Future for NextTick {
//   type Item = ();
//   type Error = ();

//   fn poll(&mut self) -> Poll<(), ()> {
//     // Polling a `NextTick` just forwards to polling if the inner promise is
//     // ready.
//     match self.inner.poll() {
//       Ok(Async::Ready(_)) => Ok(Async::Ready(())),
//       Ok(Async::NotReady) => Ok(Async::NotReady),
//       Err(_) => unreachable!(
//         "We only create NextTick with a resolved inner promise, never \
//           a rejected one, so we can't get an error here"
//       ),
//     }
//   }
// }

/*
 * Synth
 */

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
  pub fn new(data: &JsValue) -> Result<FmOsc, JsValue> {
    let ctx = web_sys::AudioContext::new()?;

    // Create our web audio objects.
    let primary = ctx.create_oscillator()?;
    let fm_osc = ctx.create_oscillator()?;
    let gain = ctx.create_gain()?;
    let fm_gain = ctx.create_gain()?;
    let analyser = ctx.create_analyser()?;

    let pdata: Vec<Vec<f32>> = getdft(data)?;
    let mut real: Vec<f32> = pdata[0][..].to_vec();
    let mut imag: Vec<f32> = pdata[1][..].to_vec();
    let customwave = ctx.create_periodic_wave(&mut real, &mut imag)?;

    // Some initial settings:
    // primary.set_type(OscillatorType::Sine);
    primary.set_periodic_wave(&customwave);
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

/* MIDI controllment */

#[wasm_bindgen]
pub struct MIDIInput {
  ctx: AudioContext,                           // the Web Audio "context" object
  // midiAccess: null,                          // the MIDIAccess object.
  osc: web_sys::OscillatorNode,                 // the single oscillator
  env: web_sys::GainNode,                       // the envelope for the single oscillator
  attack: f32,                                  // attack speed
  release: f32,                                 // release speed
  portamento: f32,                              // portamento/glide speed
  // activeNotes: [],                           // the stack of actively-pressed keys
}

#[wasm_bindgen]
impl MIDIInput {

  #[wasm_bindgen(constructor)]
  pub fn new(ctx: &AudioContext, set_note: Fn(T) -> ()) -> Result<MIDIInput, JsValue> {
    // let ctx = web_sys::AudioContext::new()?;
    // let promise = Navigator::request_midi_access(web_sys::Navigator); // ???

    // set up the basic oscillator chain, muted to begin with.
    let osc = ctx.create_oscillator()?;
    osc.frequency().set_value_at_time(110.0f32, 0.0f64);
    let env = ctx.create_gain()?;
    osc.connect_with_audio_param(&env.gain());
    env.connect_with_audio_node(&ctx.destination());
    env.gain().set_value(0.0);  // Mute the sound
    osc.start();                // Go ahead and start up the oscillator

    Ok(MIDIInput {
        ctx,
        fm,
        osc,
        env,
        attack: 0.5f32,
        release: 0.5f32,
        portamento: 0.05f32,
    })
  }

  // pub fn onMIDIInit(&mut self, midi) {
  //   midiAccess = midi;

  //   let haveAtLeastOneDevice = false;
  //   let inputs = midiAccess.inputs.values();
  //   for ( let input = inputs.next(); input && !input.done; input = inputs.next()) {
  //     input.value.onmidimessage = MIDIMessageEventHandler;
  //     haveAtLeastOneDevice = true;
  //   }

  //   if (!haveAtLeastOneDevice) {
  //     alert("No MIDI input devices present.  You're gonna have a bad time.");
  //   }
  // }

  // pub fn onMIDIReject(&mut self, err) {
  //   alert("The MIDI system failed to start.  You're gonna have a bad time.");
  // }

  pub fn MIDIMessageEventHandler(&mut self, jsval: &JsValue) {
    let data: Vec<u8> = jsval.into_serde().unwrap();
    let midisignal = data[0] & 0xf0u8;

    // Mask off the lower nibble (MIDI channel, which we don't care about)
    match midisignal {
      // if velocity != 0, this is a note-on message
      0x90u8 => if data[2] != 0 { self.noteOn(&data[1]) },
      // if velocity == 0, fall thru: it's a note-off.  MIDI's weird, y'all.
      0x80u8 => self.noteOff(&data[1]),
      _ => ()
    }
  }

  fn noteOn(&mut self, noteNumber: &u8) {
    self.fm.set_note(*noteNumber);
    // activeNotes.push( noteNumber );
    // oscillator.frequency.cancelScheduledValues(0);
    // oscillator.frequency.setTargetAtTime( frequencyFromNoteNumber(noteNumber), 0, portamento );
    // envelope.gain.cancelScheduledValues(0);
    // envelope.gain.setTargetAtTime(1.0, 0, attack);
  }

  fn noteOff(&mut self, noteNumber: &u8) {
    // var position = activeNotes.indexOf(noteNumber);
    // if (position!=-1) {
    //   activeNotes.splice(position,1);
    // }
    // if (activeNotes.length == 0) {  // shut off the envelope
    //   envelope.gain.cancelScheduledValues(0);
    //   envelope.gain.setTargetAtTime(0.0, 0, release );
    // } else {
    //   oscillator.frequency.cancelScheduledValues(0);
    //   oscillator.frequency.setTargetAtTime( frequencyFromNoteNumber(activeNotes[activeNotes.length-1]), 0, portamento );
    // }
  }

}

