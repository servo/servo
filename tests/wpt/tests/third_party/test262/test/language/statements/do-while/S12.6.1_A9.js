// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: "\"do-while\" Statement is evaluated without syntax checks"
es5id: 12.6.1_A9
description: Throwing system exception whithin a "do-while" loop
---*/

//////////////////////////////////////////////////////////////////////////////
//CHECK#1
try {
	do {
	    var x = 1; 
	    abaracadabara;
	} while(0);
	throw new Test262Error('#1: "abbracadabra" lead to throwing exception');

} catch (e) {
    if (e instanceof Test262Error) throw e;
}

if (x !== 1) {
	throw new Test262Error('#1.1: x === 1. Actual:  x ==='+ x  );
}
//
//////////////////////////////////////////////////////////////////////////////
