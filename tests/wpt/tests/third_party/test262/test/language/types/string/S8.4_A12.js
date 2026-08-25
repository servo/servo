// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Assignment to string literal calls String constructor
es5id: 8.4_A12
description: Check constructor of simple assigned variable
---*/

//////////////////////////////////////////////////////////////////////////////
//CHECK#1
var str = "rock'n'roll";
if (str.constructor !== String){
  throw new Test262Error('#1: var str = "rock\'n\'roll"; str.constructor === String. Actual: ' + (str.constructor));
}
//
//////////////////////////////////////////////////////////////////////////////
