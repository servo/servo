// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
includes: [sm/non262-Math-shell.js]
description: |
  pending
esid: pending
---*/
assert.sameValue(Math.cbrt(1), 1);
assert.sameValue(Math.cbrt(-1), -1);

assertNear(Math.cbrt(1e-300), 1e-100);
assertNear(Math.cbrt(-1e-300), -1e-100);

var cbrt_data = [
    [ Math.E, 1.3956124250860895 ], 
    [ Math.PI, 1.4645918875615231 ], 
    [ Math.LN2, 0.8849970445005177 ], 
    [ Math.SQRT2, 1.1224620483093728 ]
];

var sloppy_tolerance = 3;

for (var [x, y] of cbrt_data)
    assertNear(Math.cbrt(x), y, sloppy_tolerance);

