// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 12.10.1-2-s
description: >
  with statement in strict mode throws SyntaxError (nested function where
  container is strict)
negative:
  phase: parse
  type: SyntaxError
flags: [noStrict]
---*/

$DONOTEVALUATE();

function foo() {
  'use strict';
  function f() {
    var o = {};
    with (o) {};
  }
}
