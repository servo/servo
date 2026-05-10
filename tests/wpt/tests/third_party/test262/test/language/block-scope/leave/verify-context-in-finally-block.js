// Copyright (C) 2011 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 13.1
description: >
    verify context in finally block 1
---*/
function f() {}

(function(x) {
  try {
    let x = 'inner';
    throw 0;
  } catch(e) {

  } finally {
    f();
    assert.sameValue(x, 'outer');
  }
})('outer');
