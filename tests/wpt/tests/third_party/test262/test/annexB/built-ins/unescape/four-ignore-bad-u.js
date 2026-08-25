// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-unescape-string
es6id: B.2.1.2
description: >
    Does not transform four-character patterns that are not prefixed with the
    character "u"
info: |
    [...]
    5. Repeat, while k ≠ length,
       [...]
       a. Let c be the code unit at index k within string.
       b. If c is %, then
          i. If k ≤ length-6 and the code unit at index k+1 within string is u
             and the four code units at indices k+2, k+3, k+4, and k+5 within
             string are all hexadecimal digits, then
             1. Let c be the code unit whose value is the integer represented
                by the four hexadecimal digits at indices k+2, k+3, k+4, and
                k+5 within string.
             2. Increase k by 5.
       [...]
---*/

assert.sameValue(unescape('%U0000'), '%U0000');
assert.sameValue(unescape('%t0000'), '%t0000');
assert.sameValue(unescape('%v0000'), '%v0000');
assert.sameValue(unescape('%%0000'), '%\x0000');
