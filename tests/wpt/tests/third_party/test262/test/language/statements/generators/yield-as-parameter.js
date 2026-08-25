// Copyright (C) 2013 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
    `yield` is a reserved keyword within generator function bodies and may
    not be used as the binding identifier of a parameter.
es6id: 12.1.1
negative:
  phase: parse
  type: SyntaxError
features: [generators]
---*/

$DONOTEVALUATE();

function* g(yield) {}
