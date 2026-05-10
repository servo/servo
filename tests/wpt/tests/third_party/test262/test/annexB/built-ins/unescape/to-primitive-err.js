// Copyright (C) 2020 Qu Xing. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-unescape-string
description: If [Symbol.toPrimitive] method returned an object, it should throw a TypeError
info: |
    B.2.1.2 unescape ( string )

    1. Set string to ? ToString(string).
    ....
features: [Symbol.toPrimitive]
---*/

var obj = {
  toString() { throw new Test262Error('this should be unreachable'); },
  valueOf() { throw new Test262Error('this should be unreachable'); },
  [Symbol.toPrimitive]() { return function(){}; }
};

assert.throws(TypeError, function() {
  unescape(obj);
});
