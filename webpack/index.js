import './style.sass';

const WIDTH = 600;
const HEIGHT = 800;

let fm = null;

import './node_modules/jquery-knob';

import('./pkg')
.then(module => {

  fm = null;
  let fm2 = null;
  let inote = 50;
  let base = 50;
  let intervalid;

  const play_button = document.getElementById('play');
  play_button.addEventListener('click', event => {
    if (fm === null) {

      const len = 50;
      // let sharkFinValues = new Array(len).fill(0).map(i => sharkFin(i / len) * len);
      let sharkFinValues = new Array(250).fill(0).map((_, i) => square(i, 250) * 250);
      // let sharkFinValues = new Array(len).fill(0).map((_, i) => Math.cos(i * Math.PI/2));
      // let sharkFinValues = [
      //   -1, -0.8, -0.6, -0.4, -0.2,
      //   0,
      //   0.2, 0.4, 0.6, 0.8, 1,
      // ];

      // var ft = new DFT(sharkFinValues.length);
      // ft.forward(sharkFinValues);
      // var lfoTable = audioContext.createPeriodicWave(ft.real, ft.imag);
      // console.dir(module.getdft(sharkFinValues));

      fm = new module.FmOsc(sharkFinValues);
      fm.set_note(inote);
      fm.set_fm_frequency(0);
      fm.set_fm_amount(0);
      fm.set_gain(0.8);

      let canvasCtx = document.getElementById('spectrum').getContext('2d');
      canvasCtx.clearRect(0, 0, WIDTH, HEIGHT);

      intervalid = setInterval(() => {
        draw(canvasCtx, fm.get_buffer_length(), fm.get_analyser_data());
      }, 1);
    } else {
      fm.free();
      clearInterval(intervalid);
      intervalid = null;
      fm = null;
    }
  });

  const play_button2 = document.getElementById('play2');
  play_button2.addEventListener('click', event => {
    if (fm2 === null) {
      fm2 = new module.FmOsc();
      fm2.set_note(inote + 24);
      fm2.set_fm_frequency(0);
      fm2.set_fm_amount(0);
      fm2.set_gain(0.8);
    } else {
      fm2.free();
      fm2 = null;
    }
  });

  document.body.addEventListener('keypress', event => {
    let delta = 0;

    switch(event.key) {
      case 'z':
        delta = 1;
        break;
      case 's':
        delta = 2;
        break;
      case 'x':
        delta = 3;
        break;
      case 'd':
        delta = 4;
        break;
      case 'c':
        delta = 5;
        break;
      case 'v':
        delta = 6;
        break;
      case 'g':
        delta = 7;
        break;
      case 'b':
        delta = 8;
        break;
      case 'h':
        delta = 9;
        break;
      case 'n':
        delta = 10;
        break;
      case 'j':
        delta = 11;
        break;
      case 'm':
        delta = 12;
        break;
      case 'q':
        delta += 1;
        break;
      case '2':
        delta += 2;
        break;
      case 'w':
        delta += 3;
        break;
      case '3':
        delta += 4;
        break;
      case 'e':
        delta += 5;
        break;
      case 'r':
        delta += 6;
        break;
      case '5':
        delta += 7;
        break;
      case 't':
        delta += 8;
        break;
      case '6':
        delta += 9;
        break;
      case 'y':
        delta += 10;
        break;
      case '7':
        delta += 11;
        break;
      case 'u':
        delta += 12;
        break;

      default:
        return;
    }

    if ('q2w3er5t6y7u'.indexOf(event.key) !== -1) {
      delta += 12;
    }

    inote = base + delta;

    if (fm !== null) {
      fm.set_note(inote);
    }

    console.log(inote);
  });

  $('#primary_freq').knob({
    label: 'Primary frequency',
    value: 50,
    min: 30,
    max: 80,
    angleOffset: -125,
    angleArc: 250,
    width: 100,
    height: 100,
    'change' : value => {
      if (fm) {
        fm.set_primary_frequency(Number(value));
      }
    },
  });

  $('#fm_freq').knob({
    label: 'Modulation frequency',
    value: 0,
    min: 0,
    max: 3,
    step: 0.05,
    angleOffset: -125,
    angleArc: 250,
    width: 100,
    height: 100,
    'change' : value => {
      if (fm) {
        fm.set_fm_frequency(Number(value));
      }
    },
  });

  $('#fm_amount').knob({
    label: 'Modulation amount',
    value: 0,
    min: 0,
    max: 3,
    step: 0.05,
    angleOffset: -125,
    angleArc: 250,
    width: 100,
    height: 100,
    'change' : value => {
      if (fm) {
        fm.set_fm_amount(Number(value));
      }
    },
  });

})
.catch(console.error);

/*
 * SPECTRUM
 *
 * see https://developer.mozilla.org/en-US/docs/Web/API/Web_Audio_API/Visualizations_with_Web_Audio_API
 */

function draw(canvasCtx, bufferLength, dataArray) {
  // var drawVisual = requestAnimationFrame(draw);
  canvasCtx.fillStyle = 'rgb(200, 200, 200)';
  canvasCtx.fillRect(0, 0, WIDTH, HEIGHT);
  canvasCtx.lineWidth = 2;
  canvasCtx.strokeStyle = 'rgb(0, 0, 0)';
  canvasCtx.beginPath();
  var sliceWidth = WIDTH * 1.0 / bufferLength;
  var x = 0;
  for(var i = 0; i < bufferLength; i++) {

    var v = dataArray[i] / 128.0;
    var y = v * HEIGHT/2;

    if(i === 0) {
      canvasCtx.moveTo(x, y);
    } else {
      canvasCtx.lineTo(x, y);
    }

    x += sliceWidth;
  }
  canvasCtx.lineTo(WIDTH, HEIGHT / 2);
  canvasCtx.stroke();
};

function sharkFin(x) {
  if (x < 0) return 0;
  x = x * 2 % 2 + 0.05;
  if (x < 1) {
    return  1 + Math.log(x) / 4;
  }
  return Math.pow(-x, -2);
}

function square(x, len) {
  return x/len > 0.5 ? 1 : -1;
}
