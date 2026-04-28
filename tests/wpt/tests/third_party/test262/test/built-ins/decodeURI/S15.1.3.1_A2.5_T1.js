// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    If B1 = 11110xxx ([0xF0 - 0x0F4]), B2, B3, B4 = 10xxxxxxx ([0x80 -
    0xBF]), without [B1, B2] = [0xF0, 0x80 - 0x9F], [0xF4, 0x90 - 0xBF],
    return UTF8(B1, B2, B3, B4)
esid: sec-decodeuri-encodeduri
description: Complex tests, use RFC 3629
includes: [decimalToHexString.js]
---*/

var errorCount = 0;
var count = 0;
var indexP;
var indexO = 0;

for (var indexB1 = 0xF0; indexB1 <= 0xF4; indexB1++) {
  var hexB1 = decimalToPercentHexString(indexB1);
  for (var indexB2 = 0x80; indexB2 <= 0xBF; indexB2++) {
    if ((indexB1 === 0xF0) && (indexB2 <= 0x9F)) continue;
    if ((indexB1 === 0xF4) && (indexB2 >= 0x90)) continue;
    var hexB1_B2 = hexB1 + decimalToPercentHexString(indexB2);
    for (var indexB3 = 0x80; indexB3 <= 0xBF; indexB3++) {
      var hexB1_B2_B3 = hexB1_B2 + decimalToPercentHexString(indexB3);
      for (var indexB4 = 0x80; indexB4 <= 0xBF; indexB4++) {
        var hexB1_B2_B3_B4 = hexB1_B2_B3 + decimalToPercentHexString(indexB4);
        count++;
        var index = (indexB1 & 0x07) * 0x40000 + (indexB2 & 0x3F) * 0x1000 + (indexB3 & 0x3F) * 0x40 + (indexB4 & 0x3F);
        var L = ((index - 0x10000) & 0x03FF) + 0xDC00;
        var H = (((index - 0x10000) >> 10) & 0x03FF) + 0xD800;
        if (decodeURI(hexB1_B2_B3_B4) === String.fromCharCode(H, L)) continue;

        if (indexO === 0) {
          indexO = index;
        } else {
          if ((index - indexP) !== 1) {
            if ((indexP - indexO) !== 0) {
              var hexP = decimalToHexString(indexP);
              var hexO = decimalToHexString(indexO);
              throw new Test262Error('#' + hexO + '-' + hexP + ' ');
            }
            else {
              var hexP = decimalToHexString(indexP);
              throw new Test262Error('#' + hexP + ' ');
            }
            indexO = index;
          }
        }
        indexP = index;
        errorCount++;
      }
    }
  }
}

if (errorCount > 0) {
  if ((indexP - indexO) !== 0) {
    var hexP = decimalToHexString(indexP);
    var hexO = decimalToHexString(indexO);
    throw new Test262Error('#' + hexO + '-' + hexP + ' ');
  } else {
    var hexP = decimalToHexString(indexP);
    throw new Test262Error('#' + hexP + ' ');
  }
  throw new Test262Error('Total error: ' + errorCount + ' bad Unicode character in ' + count + ' ');
}
