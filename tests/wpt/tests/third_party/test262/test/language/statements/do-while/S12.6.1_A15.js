// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Block within a "do-while" Expression is not allowed
es5id: 12.6.1_A15
description: Using "{0}" Block as an Expression
negative:
  phase: parse
  type: SyntaxError
---*/

$DONOTEVALUATE();

//////////////////////////////////////////////////////////////////////////////
//CHECK#
do{
    ;
}while({0});
//
//////////////////////////////////////////////////////////////////////////////
