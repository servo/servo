// Copyright (C) 2011 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 13.1
description: >
    for loop block let declaration only shadows outer parameter value 2
---*/
(function(x) {
  label: for (var i = 0; i < 10; ++i) {
    let x = 'middle' + i;
    for (var j = 0; j < 10; ++j) {
      let x = 'inner' + j;
      continue label;
    }
  }
  assert.sameValue(x, 'outer');
})('outer');

