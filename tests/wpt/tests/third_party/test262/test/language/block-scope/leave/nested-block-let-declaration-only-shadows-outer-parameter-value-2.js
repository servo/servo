// Copyright (C) 2011 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 13.1
description: >
    nested block let declaration only shadows outer parameter value 2
---*/
(function(x) {
  label: {
    let x = 'middle';
    {
      let x = 'inner';
      break label;
    }
  }
  assert.sameValue(x, 'outer');
})('outer');

