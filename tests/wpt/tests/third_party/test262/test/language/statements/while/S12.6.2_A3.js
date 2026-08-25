// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    When "while" IterationStatement is evaluated, (normal, V, empty) is
    returned
es5id: 12.6.2_A3
description: Using eval
---*/

var __evaluated, __in__do;

__evaluated = eval("while (false) __in__do=1;");

//////////////////////////////////////////////////////////////////////////////
//CHECK#
if (__in__do !== undefined) {
	throw new Test262Error('#1: __in__do === undefined. Actual:  __in__do ==='+ __in__do  );
}
//
//////////////////////////////////////////////////////////////////////////////

//////////////////////////////////////////////////////////////////////////////
//CHECK#2
if (__evaluated !== undefined) {
	throw new Test262Error('#2: __evaluated === undefined. Actual:  __evaluated ==='+ __evaluated  );
}
//
//////////////////////////////////////////////////////////////////////////////
