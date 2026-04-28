// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    The production ExpressionStatement : [lookahead \notin {{, function}] Expression; is evaluated as follows:
    1. Evaluate Expression.
    2. Call GetValue(Result(1)).
    3. Return (normal, Result(2), empty)
es5id: 12.4_A2_T2
description: Checking by using eval(eval(x), where x is any string)
---*/

var x, __evaluated;

x="5+1|0===0";

__evaluated = eval(x);

//////////////////////////////////////////////////////////////////////////////
//CHECK#1
if (__evaluated !== 7) {
	throw new Test262Error('#1: __evaluated === 7. Actual:  __evaluated ==='+ __evaluated  );
}
//
//////////////////////////////////////////////////////////////////////////////

__evaluated = eval("2*"+x+">-1");

//////////////////////////////////////////////////////////////////////////////
//CHECK#2
if (__evaluated !== 11) {
	throw new Test262Error('#2: __evaluated === 11. Actual:  __evaluated ==='+ __evaluated  );
}
//
//////////////////////////////////////////////////////////////////////////////
