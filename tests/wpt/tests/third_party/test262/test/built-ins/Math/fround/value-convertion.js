// Copyright (C) 2016 The V8 Project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-math.fround
description: >
  Convert to binary32 format and than to binary64 format
info: |
  Math.fround ( x )

  [...]
  3. Let x32 be the result of converting x to a value in IEEE 754-2008 binary32
  format using roundTiesToEven.
  4. Let x64 be the result of converting x32 to a value in IEEE 754-2008
  binary64 format.
  5. Return the ECMAScript Number value corresponding to x64.
---*/

assert.sameValue(Math.fround(4294967295), 4294967296, "2**32-1");
assert.sameValue(Math.fround(4294967296), 4294967296, "2**32");
assert.sameValue(Math.fround(4294967297), 4294967296, "2**32+1");

assert.sameValue(Math.fround(0.1), 0.10000000149011612, "0.1");
assert.sameValue(Math.fround(0.2), 0.20000000298023224, "0.2");
assert.sameValue(Math.fround(0.5), 0.5, "0.5");
