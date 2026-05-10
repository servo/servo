// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    FunctionDeclaration inside the "if" Expression is evaluated as true and
    function will not be declarated
es5id: 12.5_A5
description: >
    The "if" Expression is "function __func(){throw
    "FunctionExpression";}"
---*/

//////////////////////////////////////////////////////////////////////////////
//CHECK#1
try {
	__func=__func;
	throw new Test262Error('#1: "__func=__func" lead to throwing exception');
} catch (e) {
	;
}
//
//////////////////////////////////////////////////////////////////////////////


//////////////////////////////////////////////////////////////////////////////
//CHECK#2
try {
	if(function __func(){throw "FunctionExpression";}) (function(){throw "TrueBranch"})(); else (function(){"MissBranch"})();
} catch (e) {
	if (e !== "TrueBranch") {
		throw new Test262Error('#2: Exception ==="TrueBranch". Actual:  Exception ==='+ e);
	}
}
//
//////////////////////////////////////////////////////////////////////////////

//////////////////////////////////////////////////////////////////////////////
//CHECK#3
try {
	__func=__func;
	throw new Test262Error('#3: "__func=__func" lead to throwing exception');
} catch (e) {
	;
}
//
//////////////////////////////////////////////////////////////////////////////
