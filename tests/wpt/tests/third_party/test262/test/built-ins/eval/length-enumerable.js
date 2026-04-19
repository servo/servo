// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: The length property of eval has the attribute DontEnum
esid: sec-eval-x
description: Checking use propertyIsEnumerable, for-in
---*/

//CHECK#1
if (eval.propertyIsEnumerable('length') !== false) {
  throw new Test262Error('#1: eval.propertyIsEnumerable(\'length\') === false. Actual: ' + (eval.propertyIsEnumerable('length')));
}

//CHECK#2
var result = true;
for (p in eval) {
  if (p === "length") {
    result = false;
  }
}

if (result !== true) {
  throw new Test262Error('#2: result = true; for (p in eval) { if (p === "length") result = false; };  result === true;');
}
