// Copyright (C) 2014 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 14.1
description: >
    Rest parameter cannot be followed by another named parameter
negative:
  phase: parse
  type: SyntaxError
---*/

$DONOTEVALUATE();
function f(a, ...b, c) {}
