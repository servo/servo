// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Behavior when error is thrown accessing @@match property
es6id: 21.1.3.11
info: |
    [...]
    3. If regexp is neither undefined nor null, then
       a. Let matcher be GetMethod(regexp, @@match).
       b. ReturnIfAbrupt(matcher).
features: [Symbol.match]
---*/

var obj = {};
Object.defineProperty(obj, Symbol.match, {
  get: function() {
    throw new Test262Error();
  }
});

assert.throws(Test262Error, function() {
  ''.match(obj);
});
