// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: The length property of encodeURIComponent has the attribute DontEnum
esid: sec-encodeuricomponent-uricomponent
description: Checking use propertyIsEnumerable, for-in
---*/

//CHECK#1
if (encodeURIComponent.propertyIsEnumerable('length') !== false) {
  throw new Test262Error('#1: encodeURIComponent.propertyIsEnumerable(\'length\') === false. Actual: ' + (encodeURIComponent.propertyIsEnumerable('length')));
}

//CHECK#2
var result = true;
for (var p in encodeURIComponent) {
  if (p === "length") {
    result = false;
  }
}

if (result !== true) {
  throw new Test262Error('#2: result = true; for (p in encodeURIComponent) { if (p === "length") result = false; }  result === true;');
}
