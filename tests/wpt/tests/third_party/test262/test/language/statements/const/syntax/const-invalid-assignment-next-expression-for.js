// Copyright (C) 2015 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 13.6.3.7_S5.a.i
description: >
    const: invalid assignment in next expression
---*/

assert.throws(TypeError, function() {
  for (const i = 0; i < 1; i++) {}
});
