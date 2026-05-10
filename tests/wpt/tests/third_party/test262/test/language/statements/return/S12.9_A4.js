// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    The production ReturnStatement : return Expression; is evaluated as:
    i)   Evaluate Expression.
    ii)  Call GetValue(Result(2)).
    iii) Return (return, Result(3), empty)
es5id: 12.9_A4
description: Return very sophisticated expression and function
---*/

// second derivative 
function DD_operator(f, delta){return function(x){return (f(x+delta)-2*f(x)+f(x-delta))/(delta*delta)};}

var DDsin;
DDsin = DD_operator(Math.sin, 0.00001);


//////////////////////////////////////////////////////////////////////////////
//CHECK#1
// ((sin(x))')' = -sin(x)
if (DDsin( Math.PI/2 ) + Math.sin( Math.PI/2 ) > 0.00001) {
	throw new Test262Error('#1: return Expression yields to Return (return, GetValue(Evaluate Expression), empty)');
}
//
//////////////////////////////////////////////////////////////////////////////
