// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 21.1.3.6
description: >
  Throws a TypeError if searchString is a RegExp.
info: |
  21.1.3.6 String.prototype.endsWith ( searchString [ , endPosition] )

  ...
  4. Let isRegExp be IsRegExp(searchString).
  5. ReturnIfAbrupt(isRegExp).
  6. If isRegExp is true, throw a TypeError exception.
  ...
features: [String.prototype.endsWith]
---*/

var searchString = /./;

assert.throws(TypeError, function() {
  ''.endsWith(searchString);
});
