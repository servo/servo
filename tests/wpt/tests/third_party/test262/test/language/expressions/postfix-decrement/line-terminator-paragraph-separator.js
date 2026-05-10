// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Line Terminator between LeftHandSideExpression and "--" is not allowed
es5id: 11.3.2_A1.1_T4
esid: sec-postfix-decrement-operator
description: Checking Paragraph separator
negative:
  phase: parse
  type: SyntaxError
---*/

$DONOTEVALUATE();

x --;
// The preceding line contains an unprintable PARAGRAPH SEPARATOR character
// (U+2029)
