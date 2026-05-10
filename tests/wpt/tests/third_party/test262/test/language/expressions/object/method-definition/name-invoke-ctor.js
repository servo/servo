// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
    Functions declared as methods may not be used as constructors.
es6id: 14.3.8
---*/

var obj = { method() {} };
assert.throws(TypeError, function() {
  new obj.method();
});
