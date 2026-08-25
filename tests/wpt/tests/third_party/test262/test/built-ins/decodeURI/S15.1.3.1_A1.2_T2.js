// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    If B = string.charAt(k+1) + string.charAt(k+2) do not represent
    hexadecimal digits, throw URIError
esid: sec-decodeuri-encodeduri
description: Complex tests
---*/

//CHECK
var result = true;
var interval = [
  [0x00, 0x2F],
  [0x3A, 0x40],
  [0x47, 0x60],
  [0x67, 0xFFFF]
];
for (var indexI = 0; indexI < interval.length; indexI++) {
  for (var indexJ = interval[indexI][0]; indexJ <= interval[indexI][1]; indexJ++) {
    try {
      decodeURI("%" + "1" + String.fromCharCode(indexJ));
      result = false;
    } catch (e) {
      if ((e instanceof URIError) !== true) {
        result = false;
      }
    }
  }
}

if (result !== true) {
  throw new Test262Error('#1: If string.charAt(k+2) does not represent hexadecimal digits, throw URIError');
}
