// Copyright (C) 2011 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 13.1
description: >
    let declarations with initialisers in statement positions:
    label: Statement
negative:
  phase: parse
  type: SyntaxError
---*/

$DONOTEVALUATE();
label: let x = 1;
