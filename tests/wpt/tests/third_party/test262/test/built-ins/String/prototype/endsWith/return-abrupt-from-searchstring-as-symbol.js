// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 21.1.3.6
description: >
  Returns abrupt from ToString(searchString) as a Symbol
info: |
  21.1.3.6 String.prototype.endsWith ( searchString [ , endPosition] )

  ...
  7. Let searchStr be ToString(searchString).
  8. ReturnIfAbrupt(searchStr).
  ...
features: [Symbol, String.prototype.endsWith]
---*/

var s = Symbol();

assert.throws(TypeError, function() {
  ''.endsWith(s);
});
