// Copyright (C) 2020 Alexey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-destructuring-binding-patterns-runtime-semantics-bindinginitialization
description: >
  Destructuring initializer is not evaluated when value is an object
  with [[IsHTMLDDA]] internal slot.
info: |
  BindingPattern : ArrayBindingPattern

  1. Let iteratorRecord be ? GetIterator(value).
  2. Let result be IteratorBindingInitialization of ArrayBindingPattern with arguments
  iteratorRecord and environment.
  3. If iteratorRecord.[[Done]] is false, return ? IteratorClose(iteratorRecord, result).
  4. Return result.

  Runtime Semantics: IteratorBindingInitialization

  SingleNameBinding : BindingIdentifier Initializer[opt]

  [...]
  5. If Initializer is present and v is undefined, then
    [...]
  6. If environment is undefined, return ? PutValue(lhs, v).
features: [destructuring-binding, IsHTMLDDA]
---*/

var IsHTMLDDA = $262.IsHTMLDDA;
var initCount = 0;
var counter = function() {
  initCount += 1;
};

var x;
([x = counter()] = [IsHTMLDDA]);

assert.sameValue(x, IsHTMLDDA);
assert.sameValue(initCount, 0);

var base = {};
([base.y = counter()] = [IsHTMLDDA]);

assert.sameValue(base.y, IsHTMLDDA);
assert.sameValue(initCount, 0);
