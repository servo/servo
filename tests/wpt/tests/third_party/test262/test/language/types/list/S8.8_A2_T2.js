// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Values of the List type are simply ordered sequences of values
es5id: 8.8_A2_T2
description: Call function __mFunc([,,]) with 1 arguments
---*/

function __mFunc(){return arguments.length;};
//////////////////////////////////////////////////////////////////////////////
//CHECK#1
if (__mFunc([,,]) !== 1){
  throw new Test262Error('#1: function __mFunc(){return arguments.length;}; __mFunc([,,]) === 1. Actual: ' + (__mFunc([,,])));
}
//
//////////////////////////////////////////////////////////////////////////////
