// Copyright (C) 2020 Alexey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-runtime-semantics-classdefinitionevaluation
description: >
  IsConstructor check is performed before "prototype" lookup.
  Arrow functions are not constructors (MakeConstructor is not called on them).
info: |
  ClassDefinitionEvaluation

  [...]
  5. Else,
    [...]
    d. Let superclass be ? GetValue(superclassRef).
    e. If superclass is null, then
      [...]
    f. Else if IsConstructor(superclass) is false, throw a TypeError exception.
features: [arrow-function, class, Proxy]
---*/

var fn = () => {};
Object.defineProperty(fn, "prototype", {
  get: function() {
    throw new Test262Error("`superclass.prototype` is unreachable");
  },
});

assert.throws(TypeError, function() {
  class A extends fn {}
});

var bound = (() => {}).bind();
Object.defineProperty(bound, "prototype", {
  get: function() {
    throw new Test262Error("`superclass.prototype` is unreachable");
  },
});

assert.throws(TypeError, function() {
  class C extends bound {}
});

var proxy = new Proxy(() => {}, {
  get: function() {
    throw new Test262Error("`superclass.prototype` is unreachable");
  },
});

assert.throws(TypeError, function() {
  class C extends proxy {}
});
