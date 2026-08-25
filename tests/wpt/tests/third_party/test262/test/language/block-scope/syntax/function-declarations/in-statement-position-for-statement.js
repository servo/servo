// Copyright (C) 2011 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 13.1
description: >
    function declarations in statement position:
    for ( ;;) Statement
negative:
  phase: parse
  type: SyntaxError
---*/

$DONOTEVALUATE();
for (;false;) function g() {}

