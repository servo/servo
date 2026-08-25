// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-unescape-string
es6id: B.2.1.2
description: Translation of patterns with four digits
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

assert.sameValue(unescape('%0%u00000'), '%0\x000', '%u0000');
assert.sameValue(unescape('%0%u00010'), '%0\x010', '%u0001');

assert.sameValue(unescape('%0%u00290'), '%0)0', '%002900');
assert.sameValue(unescape('%0%u002a0'), '%0*0', '%002a00');
assert.sameValue(unescape('%0%u002A0'), '%0*0', '%002A00');
assert.sameValue(unescape('%0%u002b0'), '%0+0', '%002b00');
assert.sameValue(unescape('%0%u002B0'), '%0+0', '%002B00');
assert.sameValue(unescape('%0%u002c0'), '%0,0', '%002c00');
assert.sameValue(unescape('%0%u002C0'), '%0,0', '%002C00');
assert.sameValue(unescape('%0%u002d0'), '%0-0', '%002d00');
assert.sameValue(unescape('%0%u002D0'), '%0-0', '%002D00');

assert.sameValue(unescape('%0%u00390'), '%090', '%003900');
assert.sameValue(unescape('%0%u003a0'), '%0:0', '%003A00');
assert.sameValue(unescape('%0%u003A0'), '%0:0', '%003A00');

assert.sameValue(unescape('%0%u003f0'), '%0?0', '%003f00');
assert.sameValue(unescape('%0%u003F0'), '%0?0', '%003F00');
assert.sameValue(unescape('%0%u00400'), '%0@0', '%004000');

assert.sameValue(unescape('%0%u005a0'), '%0Z0', '%005a00');
assert.sameValue(unescape('%0%u005A0'), '%0Z0', '%005A00');
assert.sameValue(unescape('%0%u005b0'), '%0[0', '%005b00');
assert.sameValue(unescape('%0%u005B0'), '%0[0', '%005B00');

assert.sameValue(unescape('%0%u005e0'), '%0^0', '%005e00');
assert.sameValue(unescape('%0%u005E0'), '%0^0', '%005E00');
assert.sameValue(unescape('%0%u005f0'), '%0_0', '%005f00');
assert.sameValue(unescape('%0%u005F0'), '%0_0', '%005F00');
assert.sameValue(unescape('%0%u00600'), '%0`0', '%006000');
assert.sameValue(unescape('%0%u00610'), '%0a0', '%006100');

assert.sameValue(unescape('%0%u007a0'), '%0z0', '%007a00');
assert.sameValue(unescape('%0%u007A0'), '%0z0', '%007A00');
assert.sameValue(unescape('%0%u007b0'), '%0{0', '%007b00');
assert.sameValue(unescape('%0%u007B0'), '%0{0', '%007B00');

assert.sameValue(unescape('%0%ufffe0'), '%0\ufffe0', '%ufffe');
assert.sameValue(unescape('%0%uFffe0'), '%0\ufffe0', '%uFffe');
assert.sameValue(unescape('%0%ufFfe0'), '%0\ufffe0', '%ufFfe');
assert.sameValue(unescape('%0%uffFe0'), '%0\ufffe0', '%uffFe');
assert.sameValue(unescape('%0%ufffE0'), '%0\ufffe0', '%ufffE');
assert.sameValue(unescape('%0%uFFFE0'), '%0\ufffe0', '%uFFFE');

assert.sameValue(unescape('%0%uffff0'), '%0\uffff0', '%uffff');
assert.sameValue(unescape('%0%uFfff0'), '%0\uffff0', '%uFfff');
assert.sameValue(unescape('%0%ufFff0'), '%0\uffff0', '%ufFff');
assert.sameValue(unescape('%0%uffFf0'), '%0\uffff0', '%uffFf');
assert.sameValue(unescape('%0%ufffF0'), '%0\uffff0', '%ufffF');
assert.sameValue(unescape('%0%uFFFF0'), '%0\uffff0', '%uFFFF');
