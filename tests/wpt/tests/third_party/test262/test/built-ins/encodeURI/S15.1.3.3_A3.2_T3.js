// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    unescapedURISet containing one instance of each character valid in
    uriUnescaped
esid: sec-encodeuri-uri
description: "Complex tests, uriUnescaped :: uriMark"
---*/

var uriMark = ["-", "_", ".", "!", "~", "*", "'", "(", ")"];
for (var indexC = 0; indexC < uriMark.length; indexC++) {
  var str = uriMark[indexC];
  if (encodeURI(str) !== str) {
    throw new Test262Error('#' + (indexC + 1) + ': unescapedURISet containing' + str);
  }
}
