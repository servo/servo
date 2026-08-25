// Copyright (C) 2011 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 13.6.0.1
description: >
    for declaration:
    identifier "let" disallowed as lefthandside expression in strict mode
flags: [onlyStrict]
negative:
  phase: parse
  type: SyntaxError
---*/

$DONOTEVALUATE();
var o = { a: 1 };
for (let in o) { }

