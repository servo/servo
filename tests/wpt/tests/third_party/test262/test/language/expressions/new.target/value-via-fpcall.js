// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-function.prototype.call
es6id: 19.2.3.3
description: Value when invoked via `Function.prototype.call`
info: |
  [...]
  5. Return ? Call(func, thisArg, argList).
features: [new.target]
---*/

var newTarget = null;

function f() {
  newTarget = new.target;
}

f.call({});

assert.sameValue(newTarget, undefined);
