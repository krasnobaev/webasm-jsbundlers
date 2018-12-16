## deployment

```bash
npm run build
node -e "require('gh-pages').publish('dist', (err) => console.log(err))"
```

## roadmap

### OSC

- [x] feat: osc amp
- [x] feat: main amp
- [ ] feat[ADSR]: initial implementation
- [ ] refactor: osc as rust class
- [ ] feat: add/remove osc
- [ ] feat: osc instantiation and mixing into main
- [ ] feat[osc]: LFO inital implementation
- [ ] feat[osc]: pan
- [ ] feat[osc]: superwave synthesis
- [ ] feat[osc]: wave-table synthesis
- [ ] feat[osc]: plucked synthesis

### routing

- [ ] feat: FM8-like matrix

### MIDI

- [ ] on-screen keyboard
- [ ] feat[input]: octave shift +1/-1
- [ ] feat[input]: MIDI controller support (main vol, +1/-1 note/octave shift)

### FX bank

- [ ] fx[reverb]: initial implementation
- [ ] fx[lp filter]: initial implementation
- [ ] fx[bp filter]: initial implementation
- [ ] fx[hp filter]: initial implementation

### note editor

- [ ] feat: step editor
- [ ] feat: .midi format import/export

### presets

- [ ] feat: store osc config in local storage
- [ ] feat: preset support import/export
- [ ] feat: bank editor
- [ ] feat: preset-randomizer
- [ ] feat: undo/redo

### VST-support

- [ ] feat: export project as VST-plugin

### …in the distance of light-year

- [ ] decentralised collaborative sequencer
- [ ] GAN-based synthesis
- [ ] …
