// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    If string.charAt(k) in [0xD800 - 0xDBFF] and string.charAt(k+1) not in
    [0xDC00 - 0xDFFF], throw URIError
esid: sec-encodeuri-uri
description: >
    Complex tests, string.charAt(k+1) in [0x0000, 0xD7FF, 0xD800,
    0xDBFE, 0xDBFF, 0xE000, 0xFFFF]
includes: [decimalToHexString.js]
---*/

var chars = [0x0000, 0xD7FF, 0xD800, 0xDBFE, 0xDBFF, 0xE000, 0xFFFF];
var errorCount = 0;
var count = 0;
var indexP;
var indexO = 0;

for (var index = 0xD800; index <= 0xDBFF; index++) {
  count++;
  var res = true;
  for (var indexC = 0; indexC < chars.length; indexC++) {
    try {
      encodeURI(String.fromCharCode(index, chars[indexC]));
    } catch (e) {
      if ((e instanceof URIError) === true) continue;
    }
    res = false;
  }
  if (res !== true) {
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
