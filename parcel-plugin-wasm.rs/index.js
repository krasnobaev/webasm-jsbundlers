import module from './lib.rs'; // or Cargo.toml
module.greet('John Doe');
alert(`a=1, b=2, a+b=${module.add(1, 2)}`);
