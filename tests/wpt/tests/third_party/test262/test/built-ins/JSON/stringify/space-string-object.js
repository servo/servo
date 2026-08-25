// Copyright (C) 2012 Ecma International. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-json.stringify
description: >
  String exotic objects are converted to primitives using ToString.
info: |
  JSON.stringify ( value [ , replacer [ , space ] ] )

  [...]
  5. If Type(space) is Object, then
    [...]
    b. Else if space has a [[StringData]] internal slot, then
      i. Set space to ? ToString(space).
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
  JSON.stringify(obj, null, new String('xxx')),
  JSON.stringify(obj, null, 'xxx')
);

var str = new String('xxx');
str.toString = function() { return '---'; };
str.valueOf = function() { throw new Test262Error('should not be called'); };

assert.sameValue(
  JSON.stringify(obj, null, str),
  JSON.stringify(obj, null, '---')
);

var abrupt = new String('xxx');
abrupt.toString = function() { throw new Test262Error(); };
abrupt.valueOf = function() { throw new Test262Error(); };

assert.throws(Test262Error, function() {
  JSON.stringify(obj, null, abrupt);
});
