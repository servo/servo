// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Behavior when error is thrown accessing @@search property
es6id: 21.1.3.15
info: |
    [...]
    3. If regexp is neither undefined nor null, then
       a. Let searcher be GetMethod(regexp, @@search).
       b. ReturnIfAbrupt(searcher).
features: [Symbol.search]
---*/

var poisonedSearch = {};
Object.defineProperty(poisonedSearch, Symbol.search, {
  get: function() {
    throw new Test262Error();
  }
});

assert.throws(Test262Error, function() {
  ''.search(poisonedSearch);
});
