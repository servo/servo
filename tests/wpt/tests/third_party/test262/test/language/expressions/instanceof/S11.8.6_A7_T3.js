// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    When "instanceof" returns true it means that
    GetValue(RelationalExpression) is constructed with ShiftExpression
es5id: 11.8.6_A7_T3
description: Checking Function object
---*/

var __func = new Function;

//CHECK#1
if (!(__func instanceof Function)) {
	throw new Test262Error('#1: If instanceof returns true then GetValue(RelationalExpression) was constructed with ShiftExpression');
}

//CHECK#2
if (__func.constructor !== Function) {
	throw new Test262Error('#2: If instanceof returns true then GetValue(RelationalExpression) was constructed with ShiftExpression');
}
