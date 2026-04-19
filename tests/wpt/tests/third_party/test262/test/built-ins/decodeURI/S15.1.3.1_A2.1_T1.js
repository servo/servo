// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: If string.charAt(k) not equal "%", return this char
esid: sec-decodeuri-encodeduri
description: Complex tests
includes: [decimalToHexString.js]
---*/

for (var indexI = 0; indexI <= 65535; indexI++) {
  if (indexI !== 0x25) {
    try {
      var str = String.fromCharCode(indexI);
      var differs = decodeURI(str) !== str;
    } catch (e) {
      throw new Test262Error('#' + decimalToHexString(indexI) + ' throws');
    }
    if (differs) {
      throw new Test262Error('#' + decimalToHexString(indexI) + ' differs');
    }
  }
}
