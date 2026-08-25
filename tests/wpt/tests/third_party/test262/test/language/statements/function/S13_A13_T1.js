// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Deleting arguments[i] leads to breaking the connection to local reference
es5id: 13_A13_T1
description: Deleting arguments[i]
---*/

function __func(__arg){
  delete arguments[0];
  if (arguments[0] !== undefined) {
    throw new Test262Error('#1.1: arguments[0] === undefined');
  }
  return __arg;
}

//////////////////////////////////////////////////////////////////////////////
//CHECK#1
if (__func(1) !== 1) {
	throw new Test262Error('#1.2: __func(1) === 1. Actual: __func(1) ==='+__func(1));
}
//
//////////////////////////////////////////////////////////////////////////////
