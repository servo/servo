// Copyright (C) 2018 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-privatefieldget
description: Static private fields not accessible via default Proxy handler
info: |
  1. Assert: P is a Private Name value.
  2. If O is not an object, throw a TypeError exception.
  3. Let entry be PrivateFieldFind(P, O).
  4. If entry is empty, throw a TypeError exception.

features: [class, class-static-fields-private]
---*/

class C {
  static #x = 1;
  static x() {
    return this.#x;
  }
}

var P = new Proxy(C, {});

assert.sameValue(C.x(), 1);
assert.throws(TypeError, function() {
  P.x();
});
