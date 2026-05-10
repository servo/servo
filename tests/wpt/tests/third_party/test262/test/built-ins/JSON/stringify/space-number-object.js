// Copyright (C) 2012 Ecma International. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-json.stringify
description: >
  Number objects are converted to primitives using ToNumber.
info: |
  JSON.stringify ( value [ , replacer [ , space ] ] )

  [...]
  5. If Type(space) is Object, then
    a. If space has a [[NumberData]] internal slot, then
      i. Set space to ? ToNumber(space).
---*/

var obj = {
  a1: {
    b1: [1, 2, 3, 4],
    b2: {
      c1: 1,
      c2: 2,
    },
  },
  a2: 'a2',
};

assert.sameValue(
  JSON.stringify(obj, null, new Number(1)),
  JSON.stringify(obj, null, 1)
);

var num = new Number(1);
num.toString = function() { throw new Test262Error('should not be called'); };
num.valueOf = function() { return 3; };

assert.sameValue(
  JSON.stringify(obj, null, num),
  JSON.stringify(obj, null, 3)
);

var abrupt = new Number(4);
abrupt.toString = function() { throw new Test262Error(); };
abrupt.valueOf = function() { throw new Test262Error(); };

assert.throws(Test262Error, function() {
  JSON.stringify(obj, null, abrupt);
});
