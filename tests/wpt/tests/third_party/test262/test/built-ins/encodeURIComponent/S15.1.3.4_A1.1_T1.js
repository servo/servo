// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: If string.charAt(k) in [0xDC00 - 0xDFFF], throw URIError
esid: sec-encodeuricomponent-uricomponent
description: Complex tests
includes: [decimalToHexString.js]
---*/

var errorCount = 0;
var count = 0;
var indexP;
var indexO = 0;

for (var index = 0xDC00; index <= 0xDFFF; index++) {
  count++;
  try {
    encodeURIComponent(String.fromCharCode(index));
  } catch (e) {
    if ((e instanceof URIError) === true) continue;
  }
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
