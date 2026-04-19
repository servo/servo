// Copyright (c) 2018 Mike Pennisi.  All rights reserved.
// Copyright (c) 2021 Gus Caplan. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-delete-operator-static-semantics-early-errors
description: Parsing error when operand is an IdentifierReference
info: |
  It is a Syntax Error if the UnaryExpression is contained in strict mode code
  and the derived UnaryExpression is PrimaryExpression:IdentifierReference.
negative:
  phase: parse
  type: SyntaxError
flags: [onlyStrict]
---*/

$DONOTEVALUATE();

delete ((identifier));
