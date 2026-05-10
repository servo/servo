// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: The length property of encodeURI has the attribute ReadOnly
esid: sec-encodeuri-uri
description: Checking if varying the length property fails
includes: [propertyHelper.js]
---*/

//CHECK#1
var x = encodeURI.length;
verifyNotWritable(encodeURI, "length", null, Infinity);
if (encodeURI.length !== x) {
  throw new Test262Error('#1: x = encodeURI.length; encodeURI.length = Infinity; encodeURI.length === x. Actual: ' + (encodeURI.length));
}
