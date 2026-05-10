// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
    Functions that throw instances of the specified constructor function
    satisfy the assertion.
---*/

function MyError() {}

assert.throws(MyError, function() {
  throw new MyError();
});
