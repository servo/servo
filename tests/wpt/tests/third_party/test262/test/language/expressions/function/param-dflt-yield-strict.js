// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-function-definitions
es6id: 14.1
description: >
  The `yield` token is interpreted as an IdentifierReference within a generator
  and outside of strict mode
info: |
  FunctionExpression :
    function BindingIdentifieropt ( FormalParameters ) { FunctionBody }
features: [generators, default-parameters]
flags: [onlyStrict]
negative:
  phase: parse
  type: SyntaxError
---*/

$DONOTEVALUATE();

function *g() {
  0, function(x = yield) {
    paramValue = x;
  };
}
