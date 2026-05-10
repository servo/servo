// Copyright (C) 2011 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 13.6.4.1
description: >
    It is a Syntax Error if the BoundNames of ForDeclaration contains "let".
negative:
  phase: parse
  type: SyntaxError
---*/

$DONOTEVALUATE();
for (let let in {}) { }

