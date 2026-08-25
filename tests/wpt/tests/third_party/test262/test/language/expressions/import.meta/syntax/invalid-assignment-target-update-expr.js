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

  12.4.1 Static Semantics: Early Errors

    UpdateExpression:
      LeftHandSideExpression++
      LeftHandSideExpression--

    It is an early Syntax Error if AssignmentTargetType of LeftHandSideExpression is invalid or strict.
flags: [module]
negative:
  phase: parse
  type: SyntaxError
features: [import.meta]
---*/

$DONOTEVALUATE();

import.meta++;
