// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: If B1 = 0xxxxxxxx ([0x00 - 0x7F]), return B1
esid: sec-decodeuricomponent-encodeduricomponent
description: Complex tests, use RFC 3629
includes: [decimalToHexString.js]
---*/

var errorCount = 0;
var count = 0;
var indexP;
var indexO = 0;
for (var indexB1 = 0x00; indexB1 <= 0x7F; indexB1++) {
  count++;
  var hexB1 = decimalToPercentHexString(indexB1);
  var index = indexB1;
  var hex = String.fromCharCode(index);
  if (decodeURIComponent(hexB1) === hex) continue;

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
