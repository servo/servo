// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    StatementList: StatementList Statement inside the Block is evaluated from
    left to right
es5id: 12.1_A5
description: Throwing exceptions within embedded/sequence Blocks
---*/

//////////////////////////////////////////////////////////////////////////////
//CHECK#1
try {
	throw 1;
    throw 2;
    throw 3;
    throw new Test262Error('1.1: throw 1 lead to throwing exception');
} catch (e) {
	if (e!==1) {
		throw new Test262Error('#1.2: Exception === 1. Actual:  Exception ==='+ e);
	}
}
////////////////////////////////////////////////////////////////////////////////

//////////////////////////////////////////////////////////////////////////////
//CHECK#2
try {
	{
	    throw 1;
        throw 2;
    }
    throw 3;
    throw new Test262Error('#2.1: throw 1 lead to throwing exception');
} catch (e) {
	if (e!==1) {
		throw new Test262Error('#2.2: Exception === 1. Actual:  Exception ==='+ e);
	}
}
////////////////////////////////////////////////////////////////////////////////

//////////////////////////////////////////////////////////////////////////////
//CHECK#3
try {
	throw 1;
    {
        throw 2;
        throw 3;
    }
    throw new Test262Error('#3.1: throw 1 lead to throwing exception');
} catch (e) {
	if (e!==1) {
		throw new Test262Error('#3.2: Exception === 1. Actual:  Exception ==='+ e);
	}
}
////////////////////////////////////////////////////////////////////////////////
