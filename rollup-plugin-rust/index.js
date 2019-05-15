import './node_modules/buffer-es6/index.js';
import instantiateWasm from './lib.rs';

async function init() {
  const { instance } = await instantiateWasm();
  return instance.exports;
}

init().then(({ add, greet }) => {
  // greet('John Doe');
  alert(`a=1, b=2, a+b=${add(1, 2)}`);
});
