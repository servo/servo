// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: The decodeURI property has the attribute DontEnum
esid: sec-decodeuri-encodeduri
description: Checking use propertyIsEnumerable, for-in
---*/

//CHECK#1
if (this.propertyIsEnumerable('decodeURI') !== false) {
  throw new Test262Error('#1: this.propertyIsEnumerable(\'decodeURI\') === false. Actual: ' + (this.propertyIsEnumerable('decodeURI')));
}

//CHECK#2
var result = true;
for (var p in this) {
  if (p === "decodeURI") {
    result = false;
  }
}

if (result !== true) {
  throw new Test262Error('#2: result = true; for (p in this) { if (p === "decodeURI") result = false; }  result === true;');
}
