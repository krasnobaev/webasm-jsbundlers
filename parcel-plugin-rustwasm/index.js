import('./lib.rs').then(module => {
  module.greet('John Doe');
  const a = 1;
  const b = 2;
  document.getElementById('sum').textContent = `a=${a}, b=${b}, a+b=${module.add(1, 2)}`;
});
