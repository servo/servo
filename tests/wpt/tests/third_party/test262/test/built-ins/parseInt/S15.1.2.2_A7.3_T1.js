// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Return sign * Result(17)
esid: sec-parseint-string-radix
description: Complex test. Check algorithm
---*/

//CHECK#
var R_digit1 = ["1", "2", "3", "4", "5", "6", "7", "8", "9", "A", "B", "C", "D", "E", "F", "G", "H", "I", "J", "K", "L", "M", "N", "O", "P", "Q", "R", "S", "T", "U", "V", "W", "X", "Y", "Z"];
var R_digit2 = ["1", "2", "3", "4", "5", "6", "7", "8", "9", "a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k", "l", "m", "n", "o", "p", "q", "r", "s", "t", "u", "v", "w", "x", "y", "z"];
for (var i = 2; i <= 36; i++) {
  for (var j = 0; j < 10; j++) {
    var str = "+";
    var sign = 1;
    if (j % 2 !== 0) {
      str = "-";
      sign = -1;
    }
    var num = 0;
    var pow = 1;
    var k0 = Math.max(2, i - j);
    for (var k = k0; k <= i; k++) {
      if (k % 2 === 0) {
        str = str + R_digit1[k - 2];
      } else {
        str = str + R_digit2[k - 2];
      }
      num = num + (i + (k0 - k) - 1) * pow;
      pow = pow * i;
    }
    assert.sameValue(parseInt(str, i), num * sign, 'parseInt("str + R_digit2[k - 2], i) must return num * sign');
  }
}
