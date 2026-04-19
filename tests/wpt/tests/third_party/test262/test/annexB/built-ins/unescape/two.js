// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-unescape-string
es6id: B.2.1.2
description: Translation of patterns with two digits
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

assert.sameValue(unescape('%0%0000'), '%0\x0000', '%00');
assert.sameValue(unescape('%0%0100'), '%0\x0100', '%01');

assert.sameValue(unescape('%0%2900'), '%0)00', '%29');
assert.sameValue(unescape('%0%2a00'), '%0*00', '%2a');
assert.sameValue(unescape('%0%2A00'), '%0*00', '%2A');
assert.sameValue(unescape('%0%2b00'), '%0+00', '%2b');
assert.sameValue(unescape('%0%2B00'), '%0+00', '%2B');
assert.sameValue(unescape('%0%2c00'), '%0,00', '%2c');
assert.sameValue(unescape('%0%2C00'), '%0,00', '%2C');
assert.sameValue(unescape('%0%2d00'), '%0-00', '%2d');
assert.sameValue(unescape('%0%2D00'), '%0-00', '%2D');

assert.sameValue(unescape('%0%3900'), '%0900', '%39');
assert.sameValue(unescape('%0%3a00'), '%0:00', '%3A');
assert.sameValue(unescape('%0%3A00'), '%0:00', '%3A');

assert.sameValue(unescape('%0%3f00'), '%0?00', '%3f');
assert.sameValue(unescape('%0%3F00'), '%0?00', '%3F');
assert.sameValue(unescape('%0%4000'), '%0@00', '%40');

assert.sameValue(unescape('%0%5a00'), '%0Z00', '%5a');
assert.sameValue(unescape('%0%5A00'), '%0Z00', '%5A');
assert.sameValue(unescape('%0%5b00'), '%0[00', '%5b');
assert.sameValue(unescape('%0%5B00'), '%0[00', '%5B');

assert.sameValue(unescape('%0%5e00'), '%0^00', '%5e');
assert.sameValue(unescape('%0%5E00'), '%0^00', '%5E');
assert.sameValue(unescape('%0%5f00'), '%0_00', '%5f');
assert.sameValue(unescape('%0%5F00'), '%0_00', '%5F');
assert.sameValue(unescape('%0%6000'), '%0`00', '%60');
assert.sameValue(unescape('%0%6100'), '%0a00', '%61');

assert.sameValue(unescape('%0%7a00'), '%0z00', '%7a');
assert.sameValue(unescape('%0%7A00'), '%0z00', '%7A');
assert.sameValue(unescape('%0%7b00'), '%0{00', '%7b');
assert.sameValue(unescape('%0%7B00'), '%0{00', '%7B');

assert.sameValue(unescape('%0%fe00'), '%0\xfe00', '%fe');
assert.sameValue(unescape('%0%Fe00'), '%0\xfe00', '%Fe');
assert.sameValue(unescape('%0%fE00'), '%0\xfe00', '%fE');
assert.sameValue(unescape('%0%FE00'), '%0\xfe00', '%FE');

assert.sameValue(unescape('%0%ff00'), '%0\xff00', '%ff');
assert.sameValue(unescape('%0%Ff00'), '%0\xff00', '%Ff');
assert.sameValue(unescape('%0%fF00'), '%0\xff00', '%fF');
assert.sameValue(unescape('%0%FF00'), '%0\xff00', '%FF');
