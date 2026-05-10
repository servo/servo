// Copyright (C) 2018 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-privatefieldget
description: Private fields not accessible via default Proxy handler
info: |
  1. Assert: P is a Private Name value.
  2. If O is not an object, throw a TypeError exception.
  3. Let entry be PrivateFieldFind(P, O).
  4. If entry is empty, throw a TypeError exception.

features: [class, class-fields-private]
---*/


var C = class {
  #x = 1;
  x() {
    return this.#x;
  }
}

var c = new C();
var p = new Proxy(c, {});

assert.sameValue(c.x(), 1);
assert.throws(TypeError, function() {
  p.x();
});
