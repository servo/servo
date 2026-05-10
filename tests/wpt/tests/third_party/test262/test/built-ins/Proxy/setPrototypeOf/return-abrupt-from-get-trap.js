// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-proxy-object-internal-methods-and-internal-slots-call-thisargument-argumentslist
description: >
  Return abrupt getting handler trap
info: |
  [[SetPrototypeOf]] (V)

  6. Let trap be ? GetMethod(handler, "setPrototypeOf").
features: [Proxy]
---*/

var handler = Object.defineProperty({}, "setPrototypeOf", {
  get: function() {
    throw new Test262Error();
  }
});

var proxy = new Proxy({}, handler);

assert.throws(Test262Error, function() {
  Object.setPrototypeOf(proxy, {});
});
