// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es6id: 11.6
description: >
    SyntaxError expected: reserved words used as Identifier
negative:
  phase: parse
  type: SyntaxError
---*/

$DONOTEVALUATE();

var catch = 123;
