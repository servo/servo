// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: First expression is evaluated first, and then second expression
es5id: 11.5.2_A2.4_T2
description: Checking with "throw"
---*/

//CHECK#1
var x = function () { throw "x"; };
var y = function () { throw "y"; };
try {
   x() / y();
   throw new Test262Error('#1.1: var x = function () { throw "x"; }; var y = function () { throw "y"; }; x() / y() throw "x". Actual: ' + (x() / y()));
} catch (e) {
   if (e === "y") {
     throw new Test262Error('#1.2: First expression is evaluated first, and then second expression');
   } else {
     if (e !== "x") {
       throw new Test262Error('#1.3: var x = function () { throw "x"; }; var y = function () { throw "y"; }; x() / y() throw "x". Actual: ' + (e));
     }
   }
}
