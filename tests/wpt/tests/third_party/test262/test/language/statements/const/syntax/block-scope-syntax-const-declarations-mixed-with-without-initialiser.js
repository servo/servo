// Copyright (C) 2011 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 13.1
description: >
    const declarations mixed: with, without initialiser
negative:
  phase: parse
  type: SyntaxError
---*/

$DONOTEVALUATE();
const x = 1, y;

