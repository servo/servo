// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    If (Evaluate Statement).type is "continue" and (Evaluate
    Statement).target is in the current label set, iteration of labeled
    "var-loop" breaks
es5id: 12.6.3_A11.1_T1
description: Using "continue" in order to continue a loop
---*/

var __str;
__str=""

for(var index=0; index<10; index+=1) {
	if (index<5)continue;
	__str+=index;
}

if (__str!=="56789") {
	throw new Test262Error('#1: __str === "56789". Actual:  __str ==='+ __str  );
}
