// Copyright (C) 2011 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 13.1
description: >
    verify context in labelled block 1
---*/
function f() {}

(function(x) {
  label: {
    let x = 'inner';
    break label;
  }
  f();  // The context could be restored from the stack after the call.
  assert.sameValue(x, 'outer');
})('outer');

