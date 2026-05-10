// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Blocks within "for(with var)" braces are not allowed
es5id: 12.6.3_A8.1_T3
description: >
    Checking if execution of "for({var index=0; index+=1;}
    index++<=10; index*2;) { arr.add(""+index);}" fails
negative:
  phase: parse
  type: SyntaxError
---*/

$DONOTEVALUATE();

var arr = [];

//////////////////////////////////////////////////////////////////////////////
//CHECK#3
for({var index=0; index+=1;} index++<=10; index*2;) {	arr.add(""+index);};
//
//////////////////////////////////////////////////////////////////////////////
