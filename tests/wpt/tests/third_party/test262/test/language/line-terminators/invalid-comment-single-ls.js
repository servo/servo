// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    Single line comments are terminated by the LINE SEPARATOR (U+2028)
    character
es5id: 7.3_A3.3_T1
description: Insert LINE SEPARATOR (\u2028) into single line comment
negative:
  phase: parse
  type: SyntaxError
---*/

$DONOTEVALUATE();

// single line LS> ??? (invalid)
