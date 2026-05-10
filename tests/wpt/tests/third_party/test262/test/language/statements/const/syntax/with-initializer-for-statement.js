// Copyright (C) 2011 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 13.1
description: >
    const declarations with initialisers in statement positions:
    for ( ;;) Statement
negative:
  phase: parse
  type: SyntaxError
---*/

$DONOTEVALUATE();
for (;false;) const x = 1;
