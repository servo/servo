// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Blocks within "for" braces are not allowed
es5id: 12.6.3_A8_T1
description: >
    Checking if execution of "for(index=0; index<100; {index++;
    index*2;}) {  arr.add(""+index);}" fails
negative:
  phase: parse
  type: SyntaxError
---*/

$DONOTEVALUATE();

var arr = [];

//////////////////////////////////////////////////////////////////////////////
//CHECK#1
for(index=0; index<100; {index++; index*2;}) {	arr.add(""+index);};
//
//////////////////////////////////////////////////////////////////////////////
