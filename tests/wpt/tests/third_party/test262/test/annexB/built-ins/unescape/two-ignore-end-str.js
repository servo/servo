// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-unescape-string
es6id: B.2.1.2
description: >
    Does not transform two-character patterns that are interrupted by the end
    of the string
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

assert.sameValue(unescape('%'), '%');
assert.sameValue(unescape('%0'), '%0');
assert.sameValue(unescape('%1'), '%1');
assert.sameValue(unescape('%2'), '%2');
assert.sameValue(unescape('%3'), '%3');
assert.sameValue(unescape('%4'), '%4');
assert.sameValue(unescape('%5'), '%5');
assert.sameValue(unescape('%6'), '%6');
assert.sameValue(unescape('%7'), '%7');
assert.sameValue(unescape('%8'), '%8');
assert.sameValue(unescape('%9'), '%9');
assert.sameValue(unescape('%a'), '%a');
assert.sameValue(unescape('%A'), '%A');
assert.sameValue(unescape('%b'), '%b');
assert.sameValue(unescape('%B'), '%B');
assert.sameValue(unescape('%c'), '%c');
assert.sameValue(unescape('%C'), '%C');
assert.sameValue(unescape('%d'), '%d');
assert.sameValue(unescape('%D'), '%D');
assert.sameValue(unescape('%e'), '%e');
assert.sameValue(unescape('%E'), '%E');
assert.sameValue(unescape('%f'), '%f');
assert.sameValue(unescape('%F'), '%F');
