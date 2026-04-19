// Copyright (C) 2017 Microsoft Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-escape-string
es6id: B.2.1.1
description: Escaping of code units above 255 from string with extended Unicode escape sequence
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
  escape('\u{10401}'), '%uD801%uDC01', '\\u{10401} => \\uD801\\uDC01 (surrogate pairs encoded in string)'
);
