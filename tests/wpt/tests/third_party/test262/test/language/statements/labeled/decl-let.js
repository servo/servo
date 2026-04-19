// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: Lexical declaration (let) not allowed in statement position
esid: sec-labelled-statements
es6id: 13.13
negative:
  phase: parse
  type: SyntaxError
---*/

$DONOTEVALUATE();

label: let x;
