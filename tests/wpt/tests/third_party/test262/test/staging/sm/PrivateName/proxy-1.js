// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  pending
esid: pending
---*/

class A {
  #x = 10;
  g() {
    return this.#x;
  }
};

var p = new Proxy(new A, {});
assert.throws(TypeError, function() {
  p.g();
});
