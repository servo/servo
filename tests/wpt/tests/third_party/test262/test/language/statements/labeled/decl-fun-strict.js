// Copyright (C) 2011 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-labelled-statements
es6id: 13.13
description: >
    function declarations in statement position in strict mode:
    label: Statement
flags: [onlyStrict]
negative:
  phase: parse
  type: SyntaxError
---*/

$DONOTEVALUATE();

label: function g() {}
