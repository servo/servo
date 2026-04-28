// Copyright (C) 2020 Alexey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-destructuring-binding-patterns-runtime-semantics-bindinginitialization
description: >
  Destructuring initializer is not evaluated when value is an object
  with [[IsHTMLDDA]] internal slot.
info: |
  BindingPattern : ObjectBindingPattern

  1. Perform ? RequireObjectCoercible(value).
  2. Return the result of performing BindingInitialization for
  ObjectBindingPattern using value and environment as arguments.

  Runtime Semantics: KeyedBindingInitialization

  SingleNameBinding : BindingIdentifier Initializer[opt]

  [...]
  4. If Initializer is present and v is undefined, then
    [...]
  5. If environment is undefined, return ? PutValue(lhs, v).
features: [destructuring-binding, IsHTMLDDA]
---*/

var initCount = 0;
var counter = function() {
  initCount += 1;
};

var x, IsHTMLDDA = $262.IsHTMLDDA;
({x = counter()} = {x: IsHTMLDDA});

assert.sameValue(x, IsHTMLDDA);
assert.sameValue(initCount, 0);
