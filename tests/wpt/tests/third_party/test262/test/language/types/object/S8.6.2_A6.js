// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    [[Construct]] constructs an object. Invoked via the new operator. Objects
    that implement this internal method are called constructors
es5id: 8.6.2_A6
description: Create a few Objects via the new operator
---*/

//////////////////////////////////////////////////////////////////////////////
//CHECK#1
var objInstance=new Object;
if (objInstance.constructor !== Object){
  throw new Test262Error('#1: var objInstance=new Object; objInstance.constructor === Object. Actual: ' + (objInstance.constructor));
}
//
//////////////////////////////////////////////////////////////////////////////

//////////////////////////////////////////////////////////////////////////////
//CHECK#2
var numInstance=new Number;
if (numInstance.constructor !== Number){
  throw new Test262Error('#2: var numInstance=new Number; numInstance.constructor === Number. Actual: ' + (numInstance.constructor));
}
//
//////////////////////////////////////////////////////////////////////////////
