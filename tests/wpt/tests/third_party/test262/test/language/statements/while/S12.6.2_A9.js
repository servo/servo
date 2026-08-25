// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: "\"while\" Statement is evaluated without syntax checks"
es5id: 12.6.2_A9
description: Throwing system exception inside "while" loop
---*/

//////////////////////////////////////////////////////////////////////////////
//CHECK#1
try {
	while(x!=1) {
	    var x = 1; 
	    abaracadabara;
	};
	throw new Test262Error('#1: "abbracadabra" lead to throwing exception');

} catch (e) {
    if (e instanceof Test262Error) throw e;
}

if (x !== 1) {
	throw new Test262Error('#1.1: while statement evaluates as is, without syntax checks');
}
//
//////////////////////////////////////////////////////////////////////////////
