import('./pkg')
.then(module => {
  let fm = null;
  let fm2 = null;
  let inote = 50;
  let base = 50;

  const play_button = document.getElementById('play');
  play_button.addEventListener('click', event => {
    if (fm === null) {
      fm = new module.FmOsc();
      fm.set_note(inote);
      fm.set_fm_frequency(0);
      fm.set_fm_amount(0);
      fm.set_gain(0.8);
    } else {
      fm.free();
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

  const primary_slider = document.getElementById('primary_input');
  primary_slider.addEventListener('input', event => {
    if (fm) {
      fm.set_note(Number(event.target.value));
    }
  });

  const fm_freq = document.getElementById('fm_freq');
  fm_freq.addEventListener('input', event => {
    if (fm) {
      fm.set_fm_frequency(Number(event.target.value));
    }
  });

  const fm_amount = document.getElementById('fm_amount');
  fm_amount.addEventListener('input', event => {
    if (fm) {
      fm.set_fm_amount(Number(event.target.value));
    }
  });
})
.catch(console.error);
