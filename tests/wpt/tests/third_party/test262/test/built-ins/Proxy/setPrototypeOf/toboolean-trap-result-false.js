// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-proxy-object-internal-methods-and-internal-slots-setprototypeof-v
description: >
  Return false if ToBoolean(trap result) is false, without checking
  target.[[IsExtensible]]
info: |
  [[SetPrototypeOf]] (V)

  8. Let booleanTrapResult be ToBoolean(? Call(trap, handler, « target, V »)).
  9. If booleanTrapResult is false, return false.
  10. Let extensibleTarget be ? IsExtensible(target).
  11. If extensibleTarget is true, return true.
features: [Proxy, Reflect, Reflect.setPrototypeOf]
---*/

var called = 0;

var target = new Proxy({}, {
  isExtensible: function() {
    called += 1;
  }
});

var p = new Proxy(target, {
  setPrototypeOf: function(t, v) {
    return v.attr;
  }
});

var result;

result = Reflect.setPrototypeOf(p, {
  attr: false
});
assert.sameValue(result, false, "false");
assert.sameValue(called, 0, "false - isExtensible is not called");

result = Reflect.setPrototypeOf(p, {
  attr: ""
});
assert.sameValue(result, false, "the empty string");
assert.sameValue(called, 0, "the empty string - isExtensible is not called");

result = Reflect.setPrototypeOf(p, {
  attr: 0
});
assert.sameValue(result, false, "0");
assert.sameValue(called, 0, "0 - isExtensible is not called");

result = Reflect.setPrototypeOf(p, {
  attr: -0
});
assert.sameValue(result, false, "-0");
assert.sameValue(called, 0, "-0 - isExtensible is not called");

result = Reflect.setPrototypeOf(p, {
  attr: null
});
assert.sameValue(result, false, "null");
assert.sameValue(called, 0, "null - isExtensible is not called");

result = Reflect.setPrototypeOf(p, {
  attr: undefined
});
assert.sameValue(result, false, "undefined");
assert.sameValue(called, 0, "undefined - isExtensible is not called");

result = Reflect.setPrototypeOf(p, {
  attr: NaN
});
assert.sameValue(result, false, "NaN");
assert.sameValue(called, 0, "NaN - isExtensible is not called");
