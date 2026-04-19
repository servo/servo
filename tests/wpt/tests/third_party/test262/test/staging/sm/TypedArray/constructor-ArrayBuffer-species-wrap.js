// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
includes: [sm/non262-TypedArray-shell.js]
description: |
  pending
esid: pending
---*/

let g = $262.createRealm().global;

// Both TypedArray and ArrayBuffer from different global.
for (let ctor of typedArrayConstructors) {
  let a = g.eval(`new ${ctor.name}([1, 2, 3, 4, 5]);`);
  for (let ctor2 of typedArrayConstructors) {
    let b = new ctor2(a);
    assert.sameValue(Object.getPrototypeOf(b).constructor, ctor2);
    assert.sameValue(Object.getPrototypeOf(b.buffer).constructor, ArrayBuffer);
  }
}

// Only ArrayBuffer from different global.
let origSpecies = Object.getOwnPropertyDescriptor(ArrayBuffer, Symbol.species);
let modSpecies = {
  get() {
    throw new Error("unexpected @@species access");
  }
};
for (let ctor of typedArrayConstructors) {
  let a = new ctor([1, 2, 3, 4, 5]);
  for (let ctor2 of typedArrayConstructors) {
    Object.defineProperty(ArrayBuffer, Symbol.species, modSpecies);
    let b = new ctor2(a);
    Object.defineProperty(ArrayBuffer, Symbol.species, origSpecies);
    assert.sameValue(Object.getPrototypeOf(b).constructor, ctor2);
    assert.sameValue(Object.getPrototypeOf(b.buffer).constructor, ArrayBuffer);
  }
}

// Only TypedArray from different global.
g.otherArrayBuffer = ArrayBuffer;
g.eval(`
var origSpecies = Object.getOwnPropertyDescriptor(ArrayBuffer, Symbol.species);
var modSpecies = {
  get() {
    throw new Error("unexpected @@species access");
  }
};
`);
for (let ctor of typedArrayConstructors) {
  let a = g.eval(`new ${ctor.name}([1, 2, 3, 4, 5]);`);
  for (let ctor2 of typedArrayConstructors) {
    g.eval(`Object.defineProperty(ArrayBuffer, Symbol.species, modSpecies);`);
    let b = new ctor2(a);
    g.eval(`Object.defineProperty(ArrayBuffer, Symbol.species, origSpecies);`);
    assert.sameValue(Object.getPrototypeOf(b).constructor, ctor2);
    assert.sameValue(Object.getPrototypeOf(b.buffer).constructor, ArrayBuffer);
  }
}

