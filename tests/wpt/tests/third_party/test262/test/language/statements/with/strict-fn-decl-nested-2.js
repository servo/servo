// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 12.10.1-3-s
description: >
  with statement in strict mode throws SyntaxError (nested strict function)
negative:
  phase: parse
  type: SyntaxError
flags: [noStrict]
---*/

$DONOTEVALUATE();

function foo() {
  function f() {
    'use strict';
    var o = {};
    with (o) {};
  }
}
