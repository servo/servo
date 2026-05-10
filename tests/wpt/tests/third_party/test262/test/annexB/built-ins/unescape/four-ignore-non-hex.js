// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-unescape-string
es6id: B.2.1.2
description: >
    Does not transform four-character patterns that contain non-hexadecimal
    digits
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

assert.sameValue(unescape('%u000%0'), '%u000%0');

assert.sameValue(unescape('%u000g0'), '%u000g0');
assert.sameValue(unescape('%u000G0'), '%u000G0');
assert.sameValue(unescape('%u00g00'), '%u00g00');
assert.sameValue(unescape('%u00G00'), '%u00G00');
assert.sameValue(unescape('%u0g000'), '%u0g000');
assert.sameValue(unescape('%u0G000'), '%u0G000');
assert.sameValue(unescape('%ug0000'), '%ug0000');
assert.sameValue(unescape('%uG0000'), '%uG0000');

assert.sameValue(unescape('%u000u0'), '%u000u0');
assert.sameValue(unescape('%u000U0'), '%u000U0');
assert.sameValue(unescape('%u00u00'), '%u00u00');
assert.sameValue(unescape('%u00U00'), '%u00U00');
assert.sameValue(unescape('%u0u000'), '%u0u000');
assert.sameValue(unescape('%u0U000'), '%u0U000');
assert.sameValue(unescape('%uu0000'), '%uu0000');
assert.sameValue(unescape('%uU0000'), '%uU0000');
