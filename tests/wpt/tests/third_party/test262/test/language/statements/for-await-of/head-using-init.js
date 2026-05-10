// Copyright (C) 2023 Ron Buckton. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iteration-statements
description: >
  ForDeclaration containing 'using' does not allow initializer.
info: |
  IterationStatement:
    for await (ForDeclaration of AssignmentExpression) Statement
negative:
  phase: parse
  type: SyntaxError
features: [async-iteration, explicit-resource-management]
---*/

$DONOTEVALUATE();

async function fn() {
  const obj = { [Symbol.dispose]() {} };
  for await (using x = obj of []) {}
}
