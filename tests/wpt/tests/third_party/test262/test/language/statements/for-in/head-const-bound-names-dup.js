// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: The head's declaration may not contain duplicate entries
negative:
  phase: parse
  type: SyntaxError
info: |
    It is a Syntax Error if the BoundNames of ForDeclaration contains any
    duplicate entries.
esid: sec-for-in-and-for-of-statements
es6id: 13.7.5
---*/

$DONOTEVALUATE();

for (const [x, x] in {}) {}
