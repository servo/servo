// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    A property can have attribute DontDelete like NaN propertie of Number
    object
es5id: 8.6.1_A3
description: Try to delete Number.NaN
flags: [noStrict]
---*/

//////////////////////////////////////////////////////////////////////////////
//CHECK#1
if (delete Number.NaN !== false){
  throw new Test262Error('#1: delete Number.NaN === false. Actual: ' + (delete Number.NaN));
};
//
//////////////////////////////////////////////////////////////////////////////

//////////////////////////////////////////////////////////////////////////////
//CHECK#2
if (typeof(Number.NaN) === "undefined"){
  throw new Test262Error('#2: delete Number.NaN; typeof(Number.NaN) !== "undefined" ');
};
//
//////////////////////////////////////////////////////////////////////////////
