// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 21.1.3.7
description: >
  Throws a TypeError if searchString is a RegExp.
info: |
  21.1.3.7 String.prototype.includes ( searchString [ , position ] )

  ...
  4. Let isRegExp be IsRegExp(searchString).
  5. ReturnIfAbrupt(isRegExp).
  6. If isRegExp is true, throw a TypeError exception.
  ...
features: [String.prototype.includes]
---*/
var searchString = /./;

assert.throws(TypeError, function() {
  ''.includes(searchString);
});
