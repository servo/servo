// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-array.prototype.copywithin
description: >
  Return abrupt from HasProperty(O, fromKey).
info: |
  22.1.3.3 Array.prototype.copyWithin (target, start [ , end ] )

  ...
  8. Let relativeStart be ToInteger(start).
  9. ReturnIfAbrupt(relativeStart).
  10. If relativeStart < 0, let from be max((len + relativeStart),0); else let
  from be min(relativeStart, len).
  ...
  17. Repeat, while count > 0
    a. Let fromKey be ToString(from).
    b. Let toKey be ToString(to).
    c. Let fromPresent be HasProperty(O, fromKey).
    d. ReturnIfAbrupt(fromPresent).
  ...
features: [Proxy]
---*/

var o = {
  '0': 42,
  length: 1
};

var p = new Proxy(o, {
  has: function() {
    throw new Test262Error();
  }
});

assert.throws(Test262Error, function() {
  Array.prototype.copyWithin.call(p, 0, 0);
});
