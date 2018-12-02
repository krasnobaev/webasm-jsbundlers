import './node_modules/buffer-es6/index.js';
import instantiateWasm from './lib.rs';

async function init() {
  const { instance } = await instantiateWasm();
  return instance.exports;
}

init().then(({ add, greet }) => {
  // greet('John Doe');
  const a = 1;
  const b = 2;
  document.getElementById('sum').textContent = `a=${a}, b=${b}, a+b=${add(a, b)}`;
});

// export default init;
