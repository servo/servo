// Copyright (c) 2018 Rick Waldron.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-update-expressions
description: >
  In non-strict code, "--eval" does not produce an early error.
info: |
  sec-identifiers-static-semantics-assignmenttargettype

    1. If this IdentifierReference is contained in strict mode code and StringValue of Identifier is "eval" or  "arguments", return strict.
    2. Return simple.

  sec-update-expressions-static-semantics-early-errors

    UpdateExpression -- UnaryExpression

    It is an early Reference Error if AssignmentTargetType of UnaryExpression is invalid.
    It is an early Syntax Error if AssignmentTargetType of UnaryExpression is strict.
flags: [noStrict]
---*/

--eval;
