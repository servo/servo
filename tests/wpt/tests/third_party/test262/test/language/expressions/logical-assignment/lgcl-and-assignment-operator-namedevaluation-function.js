// Copyright (c) 2020 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-assignment-operators-runtime-semantics-evaluation
description: NamedEvaluation of Logical And Assignment
info: |
    AssignmentExpression:
      LeftHandSideExpression &&= AssignmentExpression

    5. If IsAnonymousFunctionDefinition(AssignmentExpression) and IsIdentifierRef of LeftHandSideExpression are both true, then
      a. Let rval be NamedEvaluation of AssignmentExpression with argument GetReferencedName(lref).
features: [logical-assignment-operators]

---*/

var value = 1;
value &&= function() {};

assert.sameValue(value.name, "value", "value");
