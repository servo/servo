
// Copyright 2019 Google, Inc.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: prod-OptionalExpression
description: >
  an optional expression cannot be target of assignment
info: |
  Static Semantics: IsValidSimpleAssignmentTarget
    LeftHandSideExpression:
      OptionalExpression
    Return false.
features: [optional-chaining]
negative:
  phase: parse
  type: SyntaxError
---*/

$DONOTEVALUATE();

const obj = {};

obj?.a = 33;
