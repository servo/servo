// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Behavior when error is thrown accessing @@replace property
es6id: 21.1.3.14
info: |
    [...]
    3. If searchValue is neither undefined nor null, then
       a. Let replacer be GetMethod(searchValue, @@replace).
       b. ReturnIfAbrupt(replacer).
features: [Symbol.replace]
---*/

var poisonedReplace = {};
Object.defineProperty(poisonedReplace, Symbol.replace, {
  get: function() {
    throw new Test262Error();
  }
});

assert.throws(Test262Error, function() {
  ''.replace(poisonedReplace);
});
