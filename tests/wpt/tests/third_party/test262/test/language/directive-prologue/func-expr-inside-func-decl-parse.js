// Copyright (c) 2018 Mike Pennisi.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: use-strict-directive
es5id: 10.1.1-16-s
description: >
    Strict Mode - Function code that is part of a FunctionExpression
    is strict function code if FunctionExpression is contained in use
    strict
negative:
  phase: parse
  type: SyntaxError
flags: [noStrict]
---*/

$DONOTEVALUATE();

function testcase() {
  "use strict";
  var static;
}
