// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: Class declaration not allowed in statement position
esid: sec-if-statement
es6id: 13.6
negative:
  phase: parse
  type: SyntaxError
---*/

$DONOTEVALUATE();

if (true) class C {}
