// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-unescape-string
es6id: B.2.1.2
description: >
    Does not transform two-character patterns that contain non-hexadecimal
    digits
info: |
    [...]
    5. Repeat, while k ≠ length,
       [...]
       a. Let c be the code unit at index k within string.
       b. If c is %, then
          [...]
          ii. Else if k ≤ length-3 and the two code units at indices k+1 and
              k+2 within string are both hexadecimal digits, then
              1. Let c be the code unit whose value is the integer represented
                 by two zeroes plus the two hexadecimal digits at indices k+1
                 and k+2 within string.
              2. Increase k by 2.
       [...]
---*/

assert.sameValue(unescape('%0%0'), '%0%0');

assert.sameValue(unescape('%0g0'), '%0g0');
assert.sameValue(unescape('%0G0'), '%0G0');
assert.sameValue(unescape('%g00'), '%g00');
assert.sameValue(unescape('%G00'), '%G00');

assert.sameValue(unescape('%0u0'), '%0u0');
assert.sameValue(unescape('%0U0'), '%0U0');
assert.sameValue(unescape('%u00'), '%u00');
assert.sameValue(unescape('%U00'), '%U00');
