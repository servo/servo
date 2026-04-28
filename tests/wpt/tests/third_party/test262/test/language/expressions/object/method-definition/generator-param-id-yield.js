// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
    The BindingIdentifier of a SingleNameBinding witihn the FormalParameters of
    a GeneratorMethod may not be the `yield` keyword.
es6id: 14.4
features: [generators]
flags: [noStrict]
negative:
  phase: parse
  type: SyntaxError
---*/

$DONOTEVALUATE();

({
  *method(yield) {}
});
