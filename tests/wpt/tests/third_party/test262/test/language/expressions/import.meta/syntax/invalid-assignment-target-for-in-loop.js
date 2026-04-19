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

  13.7.5.1 Static Semantics: Early Errors
    IterationStatement:
      for ( LeftHandSideExpression in Expression ) Statement

    It is a Syntax Error if AssignmentTargetType of LeftHandSideExpression is not simple.
flags: [module]
negative:
  phase: parse
  type: SyntaxError
features: [import.meta]
---*/

$DONOTEVALUATE();

for (import.meta in null) ;
