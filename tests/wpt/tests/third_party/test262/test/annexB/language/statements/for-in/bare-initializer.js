// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-initializers-in-forin-statement-heads
description: >
    for-in heads prohibit AssignmentExpressions
negative:
  phase: parse
  type: SyntaxError
---*/

$DONOTEVALUATE();
var a;

for (a = 0 in {});
