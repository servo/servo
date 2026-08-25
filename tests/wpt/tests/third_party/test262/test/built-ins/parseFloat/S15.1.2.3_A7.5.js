// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: The parseFloat property has the attribute DontEnum
esid: sec-parsefloat-string
description: Checking use propertyIsEnumerable, for-in
---*/

//CHECK#1
if (this.propertyIsEnumerable('parseFloat') !== false) {
  throw new Test262Error('#1:this.propertyIsEnumerable(\'parseFloat\') === false. Actual: ' + (this.propertyIsEnumerable('parseFloat')));
}

//CHECK#2
var result = true;
for (var p in this) {
  if (p === "parseFloat") {
    result = false;
  }
}

if (result !== true) {
  throw new Test262Error('#2: result = true; for (p in this) { if (p === "parseFloat") result = false; }  result === true;');
}
