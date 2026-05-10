// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: "Operator x ? y : z uses GetValue"
es5id: 11.12_A2.1_T5
description: If ToBoolean(x) is true and GetBase(z) is null, return y
---*/

//CHECK#1
var y = new Object();
if ((true ? y : z) !== y) {
  throw new Test262Error('#1: var y = new Object(); (true ? y : z) === y');
}
