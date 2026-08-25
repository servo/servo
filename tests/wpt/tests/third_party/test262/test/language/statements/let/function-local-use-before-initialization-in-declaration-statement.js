// Copyright (C) 2011 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 13.1
description: >
    let: function local use before initialization in declaration statement.
    (TDZ, Temporal Dead Zone)
---*/
assert.throws(ReferenceError, function() {
  (function() {
    let x = x + 1;
  }());
});
