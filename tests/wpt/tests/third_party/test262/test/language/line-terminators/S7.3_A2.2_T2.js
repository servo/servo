// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: CARRIAGE RETURN (U+000D) within strings is not allowed
es5id: 7.3_A2.2_T2
description: Insert real CARRIAGE RETURN into string
negative:
  phase: parse
  type: SyntaxError
---*/

$DONOTEVALUATE();

//CHECK#1
"
str
ing
";
