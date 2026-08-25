// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es6id: 21.2.5.9
description: Behavior when error thrown while accessing `lastIndex` property
info: |
    [...]
    5. Let previousLastIndex be Get(rx, "lastIndex").
    6. ReturnIfAbrupt(previousLastIndex).
features: [Symbol.search]
---*/

var poisonedLastIndex = {
  get lastIndex() {
    throw new Test262Error();
  }
};

assert.throws(Test262Error, function() {
  RegExp.prototype[Symbol.search].call(poisonedLastIndex);
});
