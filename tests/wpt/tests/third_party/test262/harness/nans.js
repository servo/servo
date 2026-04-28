// Copyright (C) 2016 the V8 project authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: |
    A collection of NaN values produced from expressions that have been observed
    to create distinct bit representations on various platforms. These provide a
    weak basis for assertions regarding the consistent canonicalization of NaN
    values in Array buffers.
defines: [NaNs]
---*/

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
