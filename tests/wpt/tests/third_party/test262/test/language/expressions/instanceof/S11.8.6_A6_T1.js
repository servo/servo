// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    Only Function objects implement [[HasInstance]] and can be proper
    ShiftExpression for the "instanceof" operator consequently
es5id: 11.8.6_A6_T1
description: Checking "this" case
---*/

//CHECK#1
try{
	({}) instanceof this;
	throw new Test262Error('#1: Only Function objects implement [[HasInstance]] and consequently can be proper ShiftExpression for The instanceof operator');
}
catch(e){
  if (e instanceof TypeError !== true) {
    throw new Test262Error('#1: Only Function objects implement [[HasInstance]] and consequently can be proper ShiftExpression for The instanceof operator');
  }
}
