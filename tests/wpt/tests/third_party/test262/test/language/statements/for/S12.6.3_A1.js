// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    The "for {;;}" for Statement with empty expressions is allowed and leads
    to performing an infinite loop
es5id: 12.6.3_A1
description: Breaking an infinite loop by throwing exception
---*/

var __in__for = 0;

//////////////////////////////////////////////////////////////////////////////
//CHECK#1
try {
	for (;;){
    //__in__for++;
    if(++__in__for>100)throw 1;
}
} catch (e) {
	if (e !== 1) {
		throw new Test262Error('#1: for {;;} is admitted and leads to infinite loop');
	}
}
//
//////////////////////////////////////////////////////////////////////////////

//////////////////////////////////////////////////////////////////////////////
//CHECK#2
if (__in__for !== 101) {
	throw new Test262Error('#2: __in__for === 101. Actual:  __in__for ==='+ __in__for  );
}
//
//////////////////////////////////////////////////////////////////////////////
