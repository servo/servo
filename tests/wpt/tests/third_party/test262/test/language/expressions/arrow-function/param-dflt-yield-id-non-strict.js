// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-arrow-function-definitions
es6id: 14.2
description: >
  The `yield` token is interpreted as an IdentifierReference outside of strict
  mode and outside of generator function bodies.
info: |
  ArrowFunction[In, Yield] :

    ArrowParameters[?Yield] [no LineTerminator here] => ConciseBody[?In]
features: [default-parameters]
flags: [noStrict]
---*/

var yield = 23;
var f, paramValue;

f = (x = yield) => { paramValue = x; };

f();

assert.sameValue(paramValue, 23);
