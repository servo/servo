// Copyright 2016 The V8 project authors. All rights reserved.
// This code is governed by the license found in the LICENSE file.
/*---
esid: sec-array.prototype.indexof
description: >
  Return +0 when fromIndex is -0 and return index refers to the first position
info: |
  22.1.3.12 Array.prototype.indexOf ( searchElement [ , fromIndex ] )

  ...
  6. If n ≥ 0, then
    a. If n is -0, let k be +0; else let k be n.
  ...
---*/

assert.sameValue(1 / [true].indexOf(true, -0), +Infinity)
