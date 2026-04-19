// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
includes: [sm/non262-Math-shell.js]
description: |
  pending
esid: pending
---*/
assertNear(Math.log1p(1e-300), 1e-300);
assertNear(Math.log1p(1e-15), 9.999999999999995e-16);
assertNear(Math.log1p(1e-6), 9.999995000003334e-7);

var log1p_data = [
    [ 1.875817529344e-70, 1.875817529344e-70 ],
    [ 6.261923313140869e-30, 6.261923313140869e-30 ],
    [ 7.09962844069878e-15, 7.099628440698755e-15 ],
    [ 1.3671879628418538e-12, 1.3671879628409192e-12 ],
    [ 2.114990849122478e-10, 2.1149908488988187e-10 ],
    [ 1.6900931765206906e-8, 1.690093162238616e-8 ],
    [ 0.0000709962844069878, 0.00007099376429006658 ],
    [ 0.0016793412882520897, 0.00167793277137076 ],
    [ 0.011404608812881634, 0.011340066517988035 ],
];

for (var [x, y] of log1p_data)
    assertNear(Math.log1p(x), y);

