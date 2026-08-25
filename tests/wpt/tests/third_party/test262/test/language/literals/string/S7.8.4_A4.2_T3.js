// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: "CharacterEscapeSequnce :: NonEscapeSequence"
es5id: 7.8.4_A4.2_T3
description: "NonEscapeSequence :: ENGLISH SMALL ALPHABET"
---*/

//CHECK#a-z without b, f, n, r, t, v, x, u
var CharacterCode = [0x0061, 0x0063, 0x0064, 0x0065, 0x0067, 0x0068, 0x0069, 0x006A, 0x006B, 0x006C, 0x006D, 0x006F, 0x0070, 0x0071, 0x0073, 0x0077, 0x0079, 0x007A];
var NonEscapeCharacter = ["\a", "\c", "\d", "\e", "\g", "\h", "\i", "\j", "\k", "\l", "\m", "\o", "\p", "\q", "\s", "\w", "\y", "\z"];
for (var index = 0; index <= 17; index++) {
  if (String.fromCharCode(CharacterCode[index]) !== NonEscapeCharacter[index]) {
    throw new Test262Error('#' + NonEscapeCharacter[index] + ' ');
  }
}
