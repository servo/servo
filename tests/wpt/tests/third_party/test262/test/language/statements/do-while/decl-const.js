// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: Lexical declaration (const) not allowed in statement position
esid: sec-do-while-statement
es6id: 13.7.2
negative:
  phase: parse
  type: SyntaxError
---*/

$DONOTEVALUATE();

do const x = null; while (false)
