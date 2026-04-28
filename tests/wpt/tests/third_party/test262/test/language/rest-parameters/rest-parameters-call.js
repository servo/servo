// Copyright (C) 2014 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 14.1
description: >
    Rest parameter and Function.prototype.call
---*/
function af(...a) {
  return a.length;
}

assert.sameValue(af.call(null), 0, "`af.call(null)` returns `0`");
assert.sameValue(af.call(null, 1), 1, "`af.call(null, 1)` returns `1`");
assert.sameValue(af.call(null, 1, 2), 2, "`af.call(null, 1, 2)` returns `2`");
