// Copyright (C) 2016 The V8 Project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-math.min
description: >
  +0 is considered to be larger than -0
info: |
  Math.min ( value1, value2 , …values )

  The comparison of values to determine the smallest value is done using the
  Abstract Relational Comparison algorithm except that +0 is considered to be
  larger than -0.
---*/

assert.sameValue(Math.min(0, 0), 0, "(0, 0)");
assert.sameValue(Math.min(-0, -0), -0, "(-0, -0)");
assert.sameValue(Math.min(0, -0), -0, "(0, -0)");
assert.sameValue(Math.min(-0, 0), -0, "(-0, 0)");
assert.sameValue(Math.min(0, 0, -0), -0, "(0, 0, -0)");
