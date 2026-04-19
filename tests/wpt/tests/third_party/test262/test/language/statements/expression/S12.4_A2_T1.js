// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    The production ExpressionStatement : [lookahead \notin {{, function}] Expression; is evaluated as follows:
    1. Evaluate Expression.
    2. Call GetValue(Result(1)).
    3. Return (normal, Result(2), empty)
es5id: 12.4_A2_T1
description: Checking by using eval "(eval("x+1+x==1"))"
---*/

var x, __evaluated;

x=1;

__evaluated = eval("x+1+x==1");

//////////////////////////////////////////////////////////////////////////////
//CHECK#1
if (__evaluated !== false) {
	throw new Test262Error('#1: __evaluated === false. Actual:  __evaluated ==='+ __evaluated  );
}
//
//////////////////////////////////////////////////////////////////////////////

__evaluated = eval("1+1+1==1");

//////////////////////////////////////////////////////////////////////////////
//CHECK#2
if (__evaluated !== false) {
	throw new Test262Error('#2: __evaluated === false. Actual:  __evaluated ==='+ __evaluated  );
}
//
//////////////////////////////////////////////////////////////////////////////
