// Copyright (C) 2014 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 14.5
description: >
    class name binding in extends expression, assigned
---*/
assert.throws(ReferenceError, function() {
  var x = (class x extends x {});
});
