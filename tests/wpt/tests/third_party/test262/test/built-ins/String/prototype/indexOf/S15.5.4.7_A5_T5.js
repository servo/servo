// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: String.prototype.indexOf works properly
es5id: 15.5.4.7_A5_T5
description: Search substring from its position in the string
---*/

var TEST_STRING = new String(" !\"#$%&'()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_`abcdefghijklmnopqrstuvwxyz{|}~");

//////////////////////////////////////////////////////////////////////////////
//CHECK#1
for (var k = 0, i = 0x0020; i < 0x007d; i++, k++) {
  if (TEST_STRING.indexOf((String.fromCharCode(i) + String.fromCharCode(i + 1) + String.fromCharCode(i + 2)), k) !== k) {
    throw new Test262Error('#' + (i - 0x0020) + ': TEST_STRING.indexOf( (String.fromCharCode(' + i + ')+ String.fromCharCode(' + (i + 1) + ') + String.fromCharCode(' + (i + 2) + ')), ' + k + ' )===' + k + '. Actual: ' + TEST_STRING.indexOf((String.fromCharCode(i) + String.fromCharCode(i + 1) + String.fromCharCode(i + 2)), k));
  }
}
//
//////////////////////////////////////////////////////////////////////////////
