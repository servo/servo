// Copyright (C) 2016 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
author: Rick Waldron
esid: sec-unary-operators
description: Exponentiation Expression syntax error, `void` UnaryExpression
info: |
  ExponentiationExpression :
    UnaryExpression
    ...

  UnaryExpression :
    ...
    `void` UnaryExpression
    ...
features: [exponentiation]

negative:
  phase: parse
  type: SyntaxError
---*/

$DONOTEVALUATE();
void 1 ** 2;
