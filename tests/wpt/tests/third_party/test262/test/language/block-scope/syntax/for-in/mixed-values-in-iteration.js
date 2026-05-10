// Copyright (C) 2011 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 13.1
description: >
    Mixed values in iteration
---*/
function fn(x) {
  let a = [];
  for (let p in x) {
    a.push(function () { return p; });
  }
  let k = 0;
  for (let q in x) {
    assert.sameValue(q, a[k]());
    ++k;
  }
}
fn({a : [0], b : 1, c : {v : 1}, get d() {}, set e(x) {}});

