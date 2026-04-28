// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    If ToString([[Get]] ToString(j)) < ToString([[Get]] ToString(k)), return -1.
    If ToString([[Get]] ToString(j)) > ToString([[Get]] ToString(k)), return 1;
    return -1
esid: sec-array.prototype.sort
description: Checking ENGLISH ALPHABET
---*/

var alphabetR = ["z", "y", "x", "w", "v", "u", "t", "s", "r", "q", "p", "o", "n", "M", "L", "K", "J", "I", "H", "G", "F", "E", "D", "C", "B", "A"];
var alphabet = ["A", "B", "C", "D", "E", "F", "G", "H", "I", "J", "K", "L", "M", "n", "o", "p", "q", "r", "s", "t", "u", "v", "w", "x", "y", "z"];

alphabetR.sort();
var result = true;
for (var i = 0; i < 26; i++) {
  if (alphabetR[i] !== alphabet[i]) result = false;
}

if (result !== true) {
  throw new Test262Error('#1: CHECK ENGLISH ALPHABET');
}
