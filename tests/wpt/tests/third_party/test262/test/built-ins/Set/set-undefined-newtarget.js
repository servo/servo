// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-set-constructor
description: >
    Set ( [ iterable ] )

    When the Set function is called with optional argument iterable the following steps are taken:

    1. If NewTarget is undefined, throw a TypeError exception.
    ...

---*/

assert.throws(TypeError, function() {
  Set();
});

assert.throws(TypeError, function() {
  Set([]);
});
