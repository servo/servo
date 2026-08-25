// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es5id: 11.1.5_6-2-1-s
description: >
    Strict Mode - SyntaxError is thrown when an assignment to a
    reserved word or a future reserved word is contained in strict code
negative:
  phase: parse
  type: SyntaxError
flags: [onlyStrict]
---*/

$DONOTEVALUATE();

void {
  get x() {
    public = 42;
  }
};
