// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-proxy-object-internal-methods-and-internal-slots-setprototypeof-v
description: >
  Return true if ToBoolean(trap result) is true, and target.[[IsExtensible]] is
  true
info: |
  [[SetPrototypeOf]] (V)

  8. Let booleanTrapResult be ToBoolean(? Call(trap, handler, « target, V »)).
  9. If booleanTrapResult is false, return false.
  10. Let extensibleTarget be ? IsExtensible(target).
  11. If extensibleTarget is true, return true.
features: [Proxy, Reflect, Reflect.setPrototypeOf, Symbol]
---*/

var called;
var target = new Proxy({}, {
  isExtensible: function() {
    called += 1;
    return true;
  },
  getPrototypeOf: function() {
    throw new Test262Error("target.[[GetPrototypeOf]] is not called");
  }
});

var p = new Proxy(target, {
  setPrototypeOf: function(t, v) {
    return v.attr;
  }
});

var result;

called = 0;
result = Reflect.setPrototypeOf(p, {
  attr: true
});
assert.sameValue(result, true, "true");
assert.sameValue(called, 1, "true - isExtensible is called");

called = 0;
result = Reflect.setPrototypeOf(p, {
  attr: "false"
});
assert.sameValue(result, true, "string");
assert.sameValue(called, 1, "string - isExtensible is called");

called = 0;
result = Reflect.setPrototypeOf(p, {
  attr: 42
});
assert.sameValue(result, true, "42");
assert.sameValue(called, 1, "number - isExtensible is called");

called = 0;
result = Reflect.setPrototypeOf(p, {
  attr: p
});
assert.sameValue(result, true, "p");
assert.sameValue(called, 1, "object - isExtensible is called");

called = 0;
result = Reflect.setPrototypeOf(p, {
  attr: []
});
assert.sameValue(result, true, "[]");
assert.sameValue(called, 1, "[] - isExtensible is called");

called = 0;
result = Reflect.setPrototypeOf(p, {
  attr: Symbol(1)
});
assert.sameValue(result, true, "symbol");
assert.sameValue(called, 1, "symbol - isExtensible is called");
