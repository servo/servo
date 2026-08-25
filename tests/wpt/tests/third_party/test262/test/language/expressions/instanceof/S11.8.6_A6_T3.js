// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    Only Function objects implement [[HasInstance]] and can be proper
    ShiftExpression for the "instanceof" operator consequently
es5id: 11.8.6_A6_T3
description: Checking if RelationalExpression is function
---*/

function MyFunct(){return 0};

//CHECK#1
if (MyFunct instanceof MyFunct){
	throw new Test262Error('#1 function MyFunct(){return 0}; MyFunct instanceof MyFunct === false');
}

//CHECK#2
if (MyFunct instanceof Function !== true){
	throw new Test262Error('#2 function MyFunct(){return 0}; MyFunct instanceof Function === true');
}

//CHECK#3
if (MyFunct instanceof Object !== true){
	throw new Test262Error('#3 function MyFunct(){return 0}; MyFunct instanceof Object === true');
}
