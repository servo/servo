// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Check Throw Statement for automatic semicolon insertion
es5id: 7.9_A4
description: Try use Throw \n Expression construction
negative:
  phase: parse
  type: SyntaxError
---*/

$DONOTEVALUATE();

//CHECK#1
try {
  throw
  1;
} catch(e) {
}
