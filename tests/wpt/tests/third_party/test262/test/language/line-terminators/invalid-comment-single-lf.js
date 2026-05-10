// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Single line comments can not contain LINE FEED (U+000A) inside
es5id: 7.3_A3.1_T3
description: Insert real LINE FEED into single line comment
negative:
  phase: parse
  type: SyntaxError
---*/

$DONOTEVALUATE();

// CHECK#1
//single
line comment
