// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-set.prototype.clear
description: >
    Set.prototype.clear ( )

    1. Let S be the this value.
    2. If Type(S) is not Object, throw a TypeError exception.

---*/

assert.throws(TypeError, function() {
  Set.prototype.clear.call("");
});

assert.throws(TypeError, function() {
  var s = new Set();
  s.clear.call("");
});
