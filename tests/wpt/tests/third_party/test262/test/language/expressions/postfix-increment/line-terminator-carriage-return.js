// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Line Terminator between LeftHandSideExpression and "++" is not allowed
es5id: 11.3.1_A1.1_T2
esid: postfix-increment-operator
description: Carriage Return
negative:
  phase: parse
  type: SyntaxError
---*/

$DONOTEVALUATE();

x
++;
// The preceding line contains an unprintable CARRIAGE RETURN character (U+000D)
