// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Block "{}" in a "do-while" Expression is evaluated to true
es5id: 12.6.1_A11
description: Checking if execution of "do {} while({})" passes
---*/

do {
    var __in__do=1;
    if(__in__do)break;
} while({});

//////////////////////////////////////////////////////////////////////////////
//CHECK#1
if (__in__do !== 1) {
	throw new Test262Error('#1: "{}" in do-while expression evaluates to true');
}
//
//////////////////////////////////////////////////////////////////////////////
