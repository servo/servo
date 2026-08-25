// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Deleting arguments[i] leads to breaking the connection to local reference
es5id: 13_A13_T2
description: Changing arguments value and then deleting the argument
---*/

function __func(__arg){
  __arg = 2;
  delete arguments[0];
  if (arguments[0] !== undefined) {
    throw new Test262Error('#1.1: arguments[0] === undefined');
  }
  return __arg;
}

//////////////////////////////////////////////////////////////////////////////
//CHECK#1
if (__func(1) !== 2) {
	throw new Test262Error('#1.2: __func(1) === 2. Actual: __func(1) ==='+__func(1));
}
//
//////////////////////////////////////////////////////////////////////////////
