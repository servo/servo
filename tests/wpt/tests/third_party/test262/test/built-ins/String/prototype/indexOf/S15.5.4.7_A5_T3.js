// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: String.prototype.indexOf works properly
es5id: 15.5.4.7_A5_T3
description: Search one symbol from its position+1 in the string
---*/

var TEST_STRING = new String(" !\"#$%&'()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_`abcdefghijklmnopqrstuvwxyz{|}~");

//////////////////////////////////////////////////////////////////////////////
//CHECK#1
for (var k = 0, i = 0x0020; i < 0x007e; i++, k++) {
  if (TEST_STRING.indexOf(String.fromCharCode(i), k + 1) !== -1) {
    throw new Test262Error('#' + (i - 0x0020) + ': TEST_STRING.indexOf( String.fromCharCode(' + i + '), ' + (k + 1) + ' )===-1. Actual: ' + TEST_STRING.indexOf(String.fromCharCode(i), k + 1));
  }
}
//
//////////////////////////////////////////////////////////////////////////////
