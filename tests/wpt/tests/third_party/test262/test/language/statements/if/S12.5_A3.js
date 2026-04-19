// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    When the production "IfStatement: if ( Expression ) Statement else
    Statement" is evaluated, Expression is evaluated first
es5id: 12.5_A3
description: The Expression is "(function(){throw 1})()"
---*/

//////////////////////////////////////////////////////////////////////////////
//CHECK#1
try {
	if ((function(){throw 1})()) abracadabra
} catch (e) {
	if (e !== 1) {
		throw new Test262Error('#1: Exception === 1. Actual:  Exception ==='+ e);
	}
}
//
//////////////////////////////////////////////////////////////////////////////

//////////////////////////////////////////////////////////////////////////////
//CHECK#2
try {
	if ((function(){throw 1})()) abracadabra; else blablachat;
} catch (e) {
	if (e !== 1) {
		throw new Test262Error('#2: Exception === 1. Actual:  Exception ==='+ e);
	}
}
//
//////////////////////////////////////////////////////////////////////////////
