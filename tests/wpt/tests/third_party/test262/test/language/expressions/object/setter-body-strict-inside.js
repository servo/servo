// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es5id: 11.1.5_7-2-2-s
description: >
    Strict Mode - SyntaxError is thrown when an assignment to a
    reserved word is made in  a strict FunctionBody of a
    PropertyAssignment
negative:
  phase: parse
  type: SyntaxError
flags: [noStrict]
---*/

$DONOTEVALUATE();

void {
  set x(value) {
    "use strict";
    public = 42;
  }
};
