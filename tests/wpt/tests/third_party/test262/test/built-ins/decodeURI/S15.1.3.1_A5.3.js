// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: The length property of decodeURI has the attribute ReadOnly
esid: sec-decodeuri-encodeduri
description: Checking if varying the length property fails
includes: [propertyHelper.js]
---*/

//CHECK#1
var x = decodeURI.length;
verifyNotWritable(decodeURI, "length", null, Infinity);
if (decodeURI.length !== x) {
  throw new Test262Error('#1: x = decodeURI.length; decodeURI.length = Infinity; decodeURI.length === x. Actual: ' + (decodeURI.length));
}
