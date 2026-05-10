// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 21.1.3.18
description: >
  Returns abrupt from ToString(searchString) as a Symbol
info: |
  21.1.3.18 String.prototype.startsWith ( searchString [ , position ] )

  ...
  7. Let searchStr be ToString(searchString).
  8. ReturnIfAbrupt(searchString).
  ...
features: [Symbol]
---*/

var s = Symbol();

assert.throws(TypeError, function() {
  ''.startsWith(s);
});
