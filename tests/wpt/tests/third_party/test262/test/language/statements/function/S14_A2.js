// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: FunctionDeclaration cannot be localed inside an Expression
es5id: 14_A2
description: Declaring a function within an "if" Expression
---*/

//////////////////////////////////////////////////////////////////////////////
//CHECK#1
if (typeof f !== 'undefined') {
	throw new Test262Error('#1: typeof f === \'undefined\'. Actual:  typeof f ==='+ typeof f  );
}
//
//////////////////////////////////////////////////////////////////////////////

//////////////////////////////////////////////////////////////////////////////
//CHECK#2
if (function f(arg){
	if (arg===0)
	   return 1;
	else
	   return f(arg-1)*arg;
}(3)!==6) {
	throw new Test262Error('#2: FunctionDeclaration cannot be localed inside an Expression');
};
//
//////////////////////////////////////////////////////////////////////////////
