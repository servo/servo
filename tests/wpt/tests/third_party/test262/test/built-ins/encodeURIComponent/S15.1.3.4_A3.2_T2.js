// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    unescapedURIComponentSet containing one instance of each character valid
    in uriUnescaped
esid: sec-encodeuricomponent-uricomponent
description: "Complex tests, uriUnescaped :: DecimalDigit"
---*/

var DecimalDigit = ["0", "1", "2", "3", "4", "5", "6", "7", "8", "9"];
for (var indexC = 0; indexC < DecimalDigit.length; indexC++) {
  var str = DecimalDigit[indexC];
  if (encodeURIComponent(str) !== str) {
    throw new Test262Error('#' + (indexC + 1) + ': unescapedURISet containing' + str);
  }
}
