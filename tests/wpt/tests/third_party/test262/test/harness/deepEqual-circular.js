// Copyright 2019 Ron Buckton. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
  values compare correctly with circular references.
includes: [deepEqual.js]
---*/

var a = { x: 1 };
var b = { x: 1 };
a.a = a;
a.b = b;
b.a = b;
b.b = a;

assert.deepEqual(a, b);
