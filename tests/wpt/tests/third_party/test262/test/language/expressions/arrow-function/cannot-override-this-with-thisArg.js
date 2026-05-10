// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 14.2
description: >
    ArrowFunction `this` cannot be overridden by thisArg

    9.2.4 FunctionInitialize (F, kind, ParameterList, Body, Scope)

      ...
      9. If kind is Arrow, set the [[ThisMode]] internal slot of F to lexical.
      ...

    9.2.1.2 OrdinaryCallBindThis ( F, calleeContext, thisArgument )

      1. Let thisMode be the value of F’s [[ThisMode]] internal slot.
      2. If thisMode is lexical, return NormalCompletion(undefined).
      ...

---*/

var calls = 0;
var usurper = {};
[1].forEach(value => {
  calls++;
  assert.notSameValue(this, usurper);
}, usurper);

assert.sameValue(calls, 1);
