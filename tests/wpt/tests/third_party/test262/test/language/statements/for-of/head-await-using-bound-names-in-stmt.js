// Copyright (C) 2023 Ron Buckton. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: The body may not re-declare variables declared in the head
negative:
  phase: parse
  type: SyntaxError
info: |
    It is a Syntax Error if any element of the BoundNames of ForDeclaration
    also occurs in the VarDeclaredNames of Statement.
esid: sec-for-in-and-for-of-statements
flags: [module]
features: [explicit-resource-management]
---*/

$DONOTEVALUATE();

for (await using x of []) {
  var x;
}
