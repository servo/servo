// Copyright (C) 2014 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 14.1
description: >
    no alias arguments
includes: [compareArray.js]
---*/
function f(a, ...rest) {
  arguments[0] = 1;
  assert.sameValue(a, 3, "The value of `a` is `3`");
  arguments[1] = 2;
  assert.compareArray(rest, [4, 5]);
}
f(3, 4, 5);
