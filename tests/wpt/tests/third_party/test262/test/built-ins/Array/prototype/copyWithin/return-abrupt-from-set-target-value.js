// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-array.prototype.copywithin
description: >
  Return abrupt from setting property value - Set(O, toKey, fromVal, true).
info: |
  22.1.3.3 Array.prototype.copyWithin (target, start [ , end ] )

  ...
  5. Let relativeTarget be ToInteger(target).
  6. ReturnIfAbrupt(relativeTarget).
  7. If relativeTarget < 0, let to be max((len + relativeTarget),0); else let to
  be min(relativeTarget, len).
  ...
  17. Repeat, while count > 0
    a. Let fromKey be ToString(from).
    b. Let toKey be ToString(to).
    ...
    e. If fromPresent is true, then
      ...
      iii. Let setStatus be Set(O, toKey, fromVal, true).
      iv. ReturnIfAbrupt(setStatus).
  ...
---*/

var o = {
  '0': true,
  length: 43
};

Object.defineProperty(o, '42', {
  set: function() {
    throw new Test262Error();
  }
});

assert.throws(Test262Error, function() {
  Array.prototype.copyWithin.call(o, 42, 0);
});
