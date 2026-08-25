// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-reflect.apply
es6id: 26.1.1
description: Value when invoked via `Reflect.apply`
info: |
  [...]
  5. Return ? Call(target, thisArgument, args).
features: [new.target, Reflect]
---*/

var newTarget = null;

function f() {
  newTarget = new.target;
}

Reflect.apply(f, {}, []);

assert.sameValue(newTarget, undefined);
