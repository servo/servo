// Copyright (C) 2014 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 14.1
description: >
    Rest parameter and Function.prototype.apply
---*/
function af(...a) {
  return a.length;
}

assert.sameValue(af.apply(null, []), 0, "`af.apply(null, [])` returns `0`");
assert.sameValue(af.apply(null, [1]), 1, "`af.apply(null, [1])` returns `1`");
assert.sameValue(af.apply(null, [1, 2]), 2, "`af.apply(null, [1, 2])` returns `2`");
assert.sameValue(af.apply(null, [1, ,2]), 3, "`af.apply(null, [1, ,2])` returns `3`");
