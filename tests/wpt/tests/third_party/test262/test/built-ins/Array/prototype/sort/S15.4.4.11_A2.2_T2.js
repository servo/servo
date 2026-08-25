// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: My comparefn is inverse implementation comparefn
esid: sec-array.prototype.sort
description: Checking RUSSIAN ALPHABET
---*/

var alphabetR = ["ё", "я", "ю", "э", "ь", "ы", "ъ", "щ", "ш", "ч", "ц", "х", "ф", "у", "т", "с", "р", "П", "О", "Н", "М", "Л", "К", "Й", "И", "З", "Ж", "Е", "Д", "Г", "В", "Б", "А"];
var alphabet = ["А", "Б", "В", "Г", "Д", "Е", "Ж", "З", "И", "Й", "К", "Л", "М", "Н", "О", "П", "р", "с", "т", "у", "ф", "х", "ц", "ч", "ш", "щ", "ъ", "ы", "ь", "э", "ю", "я", "ё"];

var myComparefn = function(x, y) {
  var xS = String(x);
  var yS = String(y);
  if (xS < yS) return 1
  if (xS > yS) return -1;
  return 0;
}

alphabet.sort(myComparefn);
var result = true;
for (var i = 0; i < 26; i++) {
  if (alphabetR[i] !== alphabet[i]) result = false;
}

if (result !== true) {
  throw new Test262Error('#1: CHECK RUSSIAN ALPHABET');
}
