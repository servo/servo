// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Behavior when error is thrown accessing @@split property
es6id: 21.1.3.17
info: |
    [...]
    3. If separator is neither undefined nor null, then
       a. Let splitter be GetMethod(separator, @@split).
       b. ReturnIfAbrupt(splitter).
features: [Symbol.split]
---*/

var poisonedSplit = {};
Object.defineProperty(poisonedSplit, Symbol.split, {
  get: function() {
    throw new Test262Error();
  }
});

assert.throws(Test262Error, function() {
  ''.split(poisonedSplit);
});
