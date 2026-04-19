// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    The specification does not provide any means for a program to access
    [[class]] value except through Object.prototype.toString
es5id: 8.6.2_A3
description: Get [[class]] value except through Object.prototype.toString
---*/

var __obj={};
//////////////////////////////////////////////////////////////////////////////
//CHECK#1
if (__obj.toString() !== "[object " + 'Object' + "]"){
  throw new Test262Error('#1: var __obj={}; __obj.toString() === "[object " + \'Object\' + "]". Actual: ' + (__obj.toString()));
}
//
//////////////////////////////////////////////////////////////////////////////
