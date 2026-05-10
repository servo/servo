// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Single line comments can not contain PARAGRAPH SEPARATOR (U+2029) inside
es5id: 7.3_A3.4_T1
description: Insert PARAGRAPH SEPARATOR (\u2029) into single line comment
negative:
  phase: parse
  type: SyntaxError
---*/

$DONOTEVALUATE();

// single line PS> ??? (invalid)
