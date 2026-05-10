// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: When appears not closed double-quote program failes
es5id: 8.4_A14_T2
description: Try to create variable using 3 double-quote
negative:
  phase: parse
  type: SyntaxError
---*/

$DONOTEVALUATE();

var str = """;
