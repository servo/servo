// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 26.1.1
description: >
  Call target with thisArgument and argumentsList
info: |
  26.1.1 Reflect.apply ( target, thisArgument, argumentsList )

  ...
  4. Perform PrepareForTailCall().
  5. Return Call(target, thisArgument, args).
features: [Reflect]
---*/

var o = {};
var count = 0;
var results, args;

function fn() {
  count++;
  results = {
    thisArg: this,
    args: arguments
  };
}

Reflect.apply(fn, o, ['arg1', 2, , null]);

assert.sameValue(count, 1, 'Called target once');
assert.sameValue(results.thisArg, o, 'Called target with `o` as `this` object');
assert.sameValue(results.args.length, 4, 'Called target with 4 arguments');
assert.sameValue(results.args[0], 'arg1');
assert.sameValue(results.args[1], 2);
assert.sameValue(results.args[2], undefined);
assert.sameValue(results.args[3], null);
