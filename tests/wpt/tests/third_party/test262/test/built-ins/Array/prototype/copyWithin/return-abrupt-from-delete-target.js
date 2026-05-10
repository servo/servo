// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-array.prototype.copywithin
description: >
  Return abrupt from deleting property value on DeletePropertyOrThrow(O, toKey).
info: |
  22.1.3.3 Array.prototype.copyWithin (target, start [ , end ] )

  ...
  17. Repeat, while count > 0
    a. Let fromKey be ToString(from).
    b. Let toKey be ToString(to).
    c. Let fromPresent be HasProperty(O, fromKey).
    ...
    f. Else fromPresent is false,
      i. Let deleteStatus be DeletePropertyOrThrow(O, toKey).
      ii. ReturnIfAbrupt(deleteStatus).
  ...
---*/

var o = {
  length: 43
};

Object.defineProperty(o, '42', {
  configurable: false,
  writable: true
});

assert.throws(TypeError, function() {
  Array.prototype.copyWithin.call(o, 42, 0);
});
