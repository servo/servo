// Copyright (C) 2011 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 13.1
description: >
    In a normal for statement the iteration variable is freshly allocated for each iteration. Multi let binding
---*/
let a = [], b = [];
for (let i = 0, j = 10; i < 5; ++i, ++j) {
  a.push(function () { return i; });
  b.push(function () { return j; });
}
for (let k = 0; k < 5; ++k) {
  assert.sameValue(k, a[k]());
  assert.sameValue(k + 10, b[k]());
}
