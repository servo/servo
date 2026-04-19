// Copyright (C) 2013 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
    `yield` may not be used as the binding identifier of a function
    expression within classes.
features: [generators]
es6id: 14.1
negative:
  phase: parse
  type: SyntaxError
---*/

$DONOTEVALUATE();

class A {
  *g() {
    (function yield() {});
  }
}
