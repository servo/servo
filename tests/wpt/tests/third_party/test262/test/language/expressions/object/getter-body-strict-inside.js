// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es5id: 11.1.5_6-2-2-s
description: >
    Strict Mode - SyntaxError is thrown when an assignment to a
    reserved word or a future reserved word is made inside a strict
    mode FunctionBody of a PropertyAssignment
negative:
  phase: parse
  type: SyntaxError
flags: [noStrict]
---*/

$DONOTEVALUATE();

void {
  get x() {
    "use strict";
    public = 42;
  }
};
