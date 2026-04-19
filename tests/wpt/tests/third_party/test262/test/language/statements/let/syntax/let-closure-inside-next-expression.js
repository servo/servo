// Copyright (C) 2011 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 13.1
description: >
    let: closure inside for loop next-expression
---*/
let a = [];
for (let i = 0; i < 5; a.push(function () { return i; }), ++i) { }
for (let k = 0; k < 5; ++k) {
  assert.sameValue(k + 1, a[k]());
}
