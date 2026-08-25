// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    unescapedURISet containing one instance of each character valid in
    uriReserved
esid: sec-encodeuri-uri
description: Complex tests
---*/

var uriReserved = [";", "/", "?", ":", "@", "&", "=", "+", "$", ","];
for (var indexC = 0; indexC < uriReserved.length; indexC++) {
  var str = uriReserved[indexC];
  if (encodeURI(str) !== str) {
    throw new Test262Error('#' + (indexC + 1) + ': unescapedURISet containing' + str);
  }
}
