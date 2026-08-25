// Copyright (C) 2018 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-static-semantics-static-semantics-assignmenttargettype
description: >
  import.meta is not a valid assignment target.
info: |
  Static Semantics: AssignmentTargetType

    ImportMeta:
      import.meta

    Return invalid.

  12.15.5.1 Static Semantics: Early Errors

    DestructuringAssignmentTarget : LeftHandSideExpression

    It is a Syntax Error if LeftHandSideExpression is neither an ObjectLiteral nor an ArrayLiteral
    and AssignmentTargetType(LeftHandSideExpression) is not simple.
flags: [module]
negative:
  phase: parse
  type: SyntaxError
features: [import.meta, destructuring-assignment]
---*/

$DONOTEVALUATE();

[import.meta] = [];
