// Copyright (C) 2017 Rick Waldron, 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: >
  Including nans.js will expose:

  var NaNs = [
    NaN,
    Number.NaN,
    NaN * 0,
    0/0,
    Infinity/Infinity,
    -(0/0),
    Math.pow(-1, 0.5),
    -Math.pow(-1, 0.5),
    Number("Not-a-Number"),
  ];

includes: [nans.js]
---*/

for (var i = 0; i < NaNs.length; i++) {
  assert.sameValue(Number.isNaN(NaNs[i]), true, "index: " + i);
}
