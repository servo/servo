// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-escape-string
es6id: B.2.1.1
description: Escaping of code units above 255
info: |
    [...]
    5. Repeat, while k < length,
       a. Let char be the code unit (represented as a 16-bit unsigned integer)
          at index k within string.
       [...]
       c. Else if char ≥ 256, then
          i. Let S be a String containing six code units "%uwxyz" where wxyz
             are the code units of the four uppercase hexadecimal digits
             encoding the value of char.
       [...]
---*/

assert.sameValue(
  escape('\u0100\u0101\u0102'), '%u0100%u0101%u0102', '\\u0100\\u0101\\u0102'
);

assert.sameValue(
  escape('\ufffd\ufffe\uffff'), '%uFFFD%uFFFE%uFFFF', '\\ufffd\\ufffd\\ufffd'
);

assert.sameValue(
  escape('\ud834\udf06'), '%uD834%uDF06', '\\ud834\\udf06 (surrogate pairs)'
);
