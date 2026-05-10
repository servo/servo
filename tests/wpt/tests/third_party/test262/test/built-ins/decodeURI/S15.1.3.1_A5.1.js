// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: The length property of decodeURI has the attribute DontEnum
esid: sec-decodeuri-encodeduri
description: Checking use propertyIsEnumerable, for-in
---*/

//CHECK#1
if (decodeURI.propertyIsEnumerable('length') !== false) {
  throw new Test262Error('#1: decodeURI.propertyIsEnumerable(\'length\') === false. Actual: ' + (decodeURI.propertyIsEnumerable('length')));
}

//CHECK#2
var result = true;
for (var p in decodeURI) {
  if (p === "length") {
    result = false;
  }
}

if (result !== true) {
  throw new Test262Error('#2: result = true; for (p in decodeURI) { if (p === "length") result = false; }  result === true;');
}
