// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-functiondeclarationinstantiation
es6id: 9.2.12
description: >
  Arguments object is created even when the body contains a lexically-scoped
  binding named "arguments"
info: |
  [...]
  19. Else if "arguments" is an element of parameterNames, then
      a. Let argumentsObjectNeeded be false.
  20. Else if hasParameterExpressions is false, then
      a. If "arguments" is an element of functionNames or if "arguments" is an
         element of lexicalNames, then
         i. Let argumentsObjectNeeded be false.
  [...]
flags: [noStrict]
---*/

var args;

function f(x = args = arguments) {
  function arguments() {}
}

f();

assert.sameValue(typeof args, 'object');
assert.sameValue(args.length, 0);
