// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    While evaluating "do Statement while ( Expression )", Statement is
    evaluated first and only after it is done Expression is checked
es5id: 12.6.1_A2
description: Evaluating Statement with error Expression
---*/

var __in__do;

try {
	do __in__do = "reached"; while (abbracadabra);
	throw new Test262Error('#1: \'do __in__do = "reached"; while (abbracadabra)\' lead to throwing exception');
} catch (e) {
    if (e instanceof Test262Error) throw e;
}

//////////////////////////////////////////////////////////////////////////////
//CHECK#1
if (__in__do !== "reached") {
	throw new Test262Error('#1.1: __in__do === "reached". Actual:  __in__do ==='+ __in__do  );
}
//
//////////////////////////////////////////////////////////////////////////////
