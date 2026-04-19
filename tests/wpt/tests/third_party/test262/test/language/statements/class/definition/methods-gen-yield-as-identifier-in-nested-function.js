// Copyright (C) 2013 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
    `yield` is a reserved keyword within normal function bodies declared
    within classes.
features: [generators]
es6id: 12.1.1
negative:
  phase: parse
  type: SyntaxError
---*/

$DONOTEVALUATE();

class A {
  *g() {
    function h() {
      yield = 1;
    }
  }
}
