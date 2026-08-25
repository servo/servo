// Copyright (C) 2014 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-runtime-semantics-classdefinitionevaluation
description: >
  IsConstructor check is performed before "prototype" lookup.
  Generator functions are not constructors (MakeConstructor is not called on them).
info: |
  ClassDefinitionEvaluation

  [...]
  5. Else,
    [...]
    d. Let superclass be ? GetValue(superclassRef).
    e. If superclass is null, then
      [...]
    f. Else if IsConstructor(superclass) is false, throw a TypeError exception.
features: [generators, class, Proxy]
---*/

function* fn() {}

assert.throws(TypeError, function() {
  class A extends fn {}
});

var bound = (function* () {}).bind();
Object.defineProperty(bound, "prototype", {
  get: function() {
    throw new Test262Error("`superclass.prototype` is unreachable");
  },
});

assert.throws(TypeError, function() {
  class C extends bound {}
});

var proxy = new Proxy(function* () {}, {});

assert.throws(TypeError, function() {
  class C extends proxy {}
});
