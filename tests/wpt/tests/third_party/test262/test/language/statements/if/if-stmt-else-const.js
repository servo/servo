// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: Lexical declaration (const) not allowed in statement position
esid: sec-if-statement
es6id: 13.6
negative:
  phase: parse
  type: SyntaxError
---*/

$DONOTEVALUATE();

if (false) ; else const x = null;
