// Copyright (c) 2018 Mike Pennisi.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 12.2.1-23-s
esid: sec-variable-statement
description: >
    arguments as local var identifier assigned to throws SyntaxError
    in strict mode within a function declaration
flags: [onlyStrict]
negative:
  phase: parse
  type: SyntaxError
---*/

$DONOTEVALUATE();

function f() {
  var arguments = 42;
}
