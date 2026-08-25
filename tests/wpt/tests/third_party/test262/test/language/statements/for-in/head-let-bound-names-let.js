// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: The declaration may not contain a binding for `let`
negative:
  phase: parse
  type: SyntaxError
info: |
    It is a Syntax Error if the BoundNames of ForDeclaration contains "let".
flags: [noStrict]
esid: sec-for-in-and-for-of-statements
es6id: 13.7.5
---*/

$DONOTEVALUATE();

for (let let in {}) {}
