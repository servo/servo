// Copyright (C) 2011 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 13.1
description: >
    verify context in for loop block 2
---*/
function f() {}

(function(x) {
  for (var i = 0; i < 10; ++i) {
    let x = 'inner';
    continue;
  }
  f();
  assert.sameValue(x, 'outer');
})('outer');

