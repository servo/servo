// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Empty string has type string
es5id: 8.4_A2
description: Create empty string and check it type
---*/

/////////////////////////////////////////////////////////
// CHECK#1
var str = '';
if (typeof(str) !== 'string'){
  throw new Test262Error('#1: var str = \'\'; typeof(str) === \'string\'. Actual: ' + (typeof(str)));
}
//
////////////////////////////////////////////////////////

/////////////////////////////////////////////////////////
// CHECK#2
var str = "";
if (typeof(str) !== "string"){
  throw new Test262Error('#2: var str = ""; typeof(str) === "string". Actual: ' + (str));
}
//
////////////////////////////////////////////////////////
