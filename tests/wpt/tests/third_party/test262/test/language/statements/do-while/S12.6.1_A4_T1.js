// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    "break" within a "do-while" Statement is allowed and performed as
    described in 12.8
es5id: 12.6.1_A4_T1
description: Using "break" within a "do-while" loop
---*/

var __in__do__before__break, __in__do__after__break;

do {
    __in__do__before__break="reached"; 
    break; 
    __in__do__after__break="where am i";
} while(2===1);

//////////////////////////////////////////////////////////////////////////////
//CHECK#1
if (__in__do__before__break !== "reached") {
	throw new Test262Error('#1: __in__do__before__break === "reached". Actual:  __in__do__before__break ==='+ __in__do__before__break  );
}
//
//////////////////////////////////////////////////////////////////////////////

//////////////////////////////////////////////////////////////////////////////
//CHECK#2
if (typeof __in__do__after__break !== "undefined") {
	throw new Test262Error('#2: typeof __in__do__after__break === "undefined". Actual:  typeof __in__do__after__break ==='+ typeof __in__do__after__break  );
}
//
//////////////////////////////////////////////////////////////////////////////
