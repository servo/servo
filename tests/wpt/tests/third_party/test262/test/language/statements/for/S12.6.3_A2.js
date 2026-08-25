// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    While evaluating "for (ExpressionNoIn; Expression; Expression)
    Statement", ExpressionNoIn is evaulated first
es5id: 12.6.3_A2
description: Using "(function(){throw "NoInExpression"})()" as ExpressionNoIn
---*/

//////////////////////////////////////////////////////////////////////////////
//CHECK#1
try {
	for((function(){throw "NoInExpression";})(); (function(){throw "FirstExpression";})(); (function(){throw "SecondExpression";})()) {
		var in_for = "reached";
	}
	throw new Test262Error('#1: (function(){throw "NoInExpression";})() lead to throwing exception');
} catch (e) {
	if (e !== "NoInExpression") {
		throw new Test262Error('#1: When for (ExpressionNoIn ; Expression ; Expression) Statement is evaluated ExpressionNoIn evaluates first');
	}
}
//
//////////////////////////////////////////////////////////////////////////////

//////////////////////////////////////////////////////////////////////////////
//CHECK#2
if (in_for !== undefined) {
	throw new Test262Error('#2: in_for === undefined. Actual:  in_for ==='+ in_for  );
}
//
//////////////////////////////////////////////////////////////////////////////
