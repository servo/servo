// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es6id: 19.2.3.2
description: Error thrown when accessing target's `name` property
info: |
    12. Let targetName be Get(Target, "name").
    13. ReturnIfAbrupt(targetName).
---*/

var target = Object.defineProperty(function() {}, 'name', {
  get: function() {
    throw new Test262Error();
  }
});

assert.throws(Test262Error, function() {
  target.bind();
});
