// Copyright (C) 2014 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 14.5
description: >
    class name binding in extends expression, grouped
---*/

assert.throws(ReferenceError, function() {
  (class x extends x {});
});
