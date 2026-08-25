// Copyright (C) 2019 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%typedarray%.prototype.sort
description: Stability of %TypedArray%.prototype.sort.
info: |
  https://github.com/tc39/ecma262/pull/1433
includes: [testTypedArray.js, compareArray.js]
features: [TypedArray, stable-typedarray-sort]
---*/

// Treat 0..3, 4..7, etc. as equal.
const compare = (a, b) => (a / 4 | 0) - (b / 4 | 0);

testWithTypedArrayConstructors((TA, makeCtorArg) => {
  // Create an array of the form `[0, 1, …, 126, 127]`.
  const array = Array.from({ length: 128 }, (_, i) => i);

  const typedArray1 = new TA(makeCtorArg(array));
  assert(compareArray(
    typedArray1.sort(compare),
    array
  ), 'pre-sorted');

  // Reverse `array` in-place so it becomes `[127, 126, …, 1, 0]`.
  array.reverse();

  const typedArray2 = new TA(makeCtorArg(array));
  assert(compareArray(
    typedArray2.sort(compare),
    [
        3,   2,   1,   0,     7,   6,   5,   4,    11,  10,   9,   8,
       15,  14,  13,  12,    19,  18,  17,  16,    23,  22,  21,  20,
       27,  26,  25,  24,    31,  30,  29,  28,    35,  34,  33,  32,
       39,  38,  37,  36,    43,  42,  41,  40,    47,  46,  45,  44,
       51,  50,  49,  48,    55,  54,  53,  52,    59,  58,  57,  56,
       63,  62,  61,  60,    67,  66,  65,  64,    71,  70,  69,  68,
       75,  74,  73,  72,    79,  78,  77,  76,    83,  82,  81,  80,
       87,  86,  85,  84,    91,  90,  89,  88,    95,  94,  93,  92,
       99,  98,  97,  96,   103, 102, 101, 100,   107, 106, 105, 104,
      111, 110, 109, 108,   115, 114, 113, 112,   119, 118, 117, 116,
      123, 122, 121, 120,   127, 126, 125, 124,
    ]
  ), 'not presorted');
}, null, null, ["immutable"]);
