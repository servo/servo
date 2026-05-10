// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 21.1.2.4
description: >
  Returns abrupt from nextKey.
info: |
  21.1.2.4 String.raw ( template , ...substitutions )

  ...
  10. Let stringElements be a new List.
  11. Let nextIndex be 0.
  12. Repeat
    a. Let nextKey be ToString(nextIndex).
    b. Let nextSeg be ToString(Get(raw, nextKey)).
    c. ReturnIfAbrupt(nextSeg).
  ...
---*/

var obj = {
  raw: {
    length: 2
  }
};

Object.defineProperty(obj.raw, '0', {
  get: function() {
    throw new Test262Error();
  },
  configurable: true
});

assert.throws(Test262Error, function() {
  String.raw(obj);
});

delete obj.raw['0'];
obj.raw['0'] = 'a';
Object.defineProperty(obj.raw, '1', {
  get: function() {
    throw new Test262Error();
  }
});
assert.throws(Test262Error, function() {
  String.raw(obj);
});
