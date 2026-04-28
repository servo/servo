// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: The encodeURI property has the attribute DontEnum
esid: sec-encodeuri-uri
description: Checking use propertyIsEnumerable, for-in
---*/

//CHECK#1
if (this.propertyIsEnumerable('encodeURI') !== false) {
  throw new Test262Error('#1: this.propertyIsEnumerable(\'encodeURI\') === false. Actual: ' + (this.propertyIsEnumerable('encodeURI')));
}

//CHECK#2
var result = true;
for (var p in this) {
  if (p === "encodeURI") {
    result = false;
  }
}

if (result !== true) {
  throw new Test262Error('#2: result = true; for (p in this) { if (p === "encodeURI") result = false; }  result === true;');
}
