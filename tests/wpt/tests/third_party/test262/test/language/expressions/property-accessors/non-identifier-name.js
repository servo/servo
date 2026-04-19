// Copyright (C) 2017 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-left-hand-side-expressions
info: |
  Token following DOT must be a valid identifier-name, test with string literal.
description: >
  12.3 Left-Hand-Side Expressions
    MemberExpression[Yield, Await]:
      MemberExpression[?Yield, ?Await] . IdentifierName

negative:
  phase: parse
  type: SyntaxError
---*/

$DONOTEVALUATE();

unresolvableReference."";
