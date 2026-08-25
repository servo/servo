// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  pending
esid: pending
---*/
//
// Ensure PrivateNames aren't iterable.

class O {
  #x = 123;
  gx() {
    return this.#x;
  }
}
var o = new O;

assert.sameValue(o.gx(), 123);

assert.sameValue(Object.keys(o).length, 0);
assert.sameValue(Object.getOwnPropertyNames(o).length, 0);
assert.sameValue(Object.getOwnPropertySymbols(o).length, 0);
assert.sameValue(Reflect.ownKeys(o).length, 0);

var forIn = [];
for (var pk in o) {
  forIn.push(pk);
}
assert.sameValue(forIn.length, 0);

// Proxy case
var proxy = new Proxy(o, {});
assert.sameValue(Object.keys(proxy).length, 0);
assert.sameValue(Object.getOwnPropertyNames(proxy).length, 0);
assert.sameValue(Object.getOwnPropertySymbols(proxy).length, 0);
assert.sameValue(Reflect.ownKeys(proxy).length, 0);

for (var pk in proxy) {
  forIn.push(pk);
}
assert.sameValue(forIn.length, 0);

