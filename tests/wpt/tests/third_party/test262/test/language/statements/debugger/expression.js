// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: The `debugger` token may not occupy an expression position
esid: sec-debugger-statement
es6id: 13.16
negative:
  phase: parse
  type: SyntaxError
---*/

$DONOTEVALUATE();

(debugger);
