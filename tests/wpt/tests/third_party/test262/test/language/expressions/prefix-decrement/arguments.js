// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-update-expressions
description: >
  It is an early Syntax Error if AssignmentTargetType of UnaryExpression is strict. (arguments)
info: |
  sec-identifiers-static-semantics-assignmenttargettype

    If this IdentifierReference is contained in strict mode code and StringValue of Identifier is "eval" or  "arguments", return strict.

  sec-update-expressions-static-semantics-early-errors

    UpdateExpression: -- UnaryExpression

    It is an early Syntax Error if AssignmentTargetType of UnaryExpression is strict.
flags: [onlyStrict]
negative:
  phase: parse
  type: SyntaxError
---*/

$DONOTEVALUATE();

--arguments;
