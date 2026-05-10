// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    If B1 = 1110xxxx ([0xE0 - 0xEF]), B2, B3 = 10xxxxxxx ([0x80 - 0xBF]),
    without [B1, B2] = [0xE0, 0x80 - 0x9F], [0xED, 0xA0 - 0xBF] (0xD800 -
    0xDFFF), return UTF8(B1, B2, B3)
esid: sec-decodeuri-encodeduri
description: Complex tests, use RFC 3629
includes: [decimalToHexString.js]
---*/

var errorCount = 0;
var count = 0;
var indexP;
var indexO = 0;

for (var indexB1 = 0xE0; indexB1 <= 0xEF; indexB1++) {
  var hexB1 = decimalToPercentHexString(indexB1);
  for (var indexB2 = 0x80; indexB2 <= 0xBF; indexB2++) {
    if ((indexB1 === 0xE0) && (indexB2 <= 0x9F)) continue;
    if ((indexB1 === 0xED) && (0xA0 <= indexB2)) continue;
    var hexB1_B2 = hexB1 + decimalToPercentHexString(indexB2);
    for (var indexB3 = 0x80; indexB3 <= 0xBF; indexB3++) {
      count++;
      var hexB1_B2_B3 = hexB1_B2 + decimalToPercentHexString(indexB3);
      var index = (indexB1 & 0x0F) * 0x1000 + (indexB2 & 0x3F) * 0x40 + (indexB3 & 0x3F);
      if (decodeURI(hexB1_B2_B3) === String.fromCharCode(index)) continue;

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
