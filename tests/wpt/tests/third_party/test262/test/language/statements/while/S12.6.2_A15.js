// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Block within a "while" Expression is not allowed
es5id: 12.6.2_A15
description: Expression is "{0}"
negative:
  phase: parse
  type: SyntaxError
---*/

$DONOTEVALUATE();

//////////////////////////////////////////////////////////////////////////////
//CHECK#
while({1}){
   break ;
};
//
//////////////////////////////////////////////////////////////////////////////
