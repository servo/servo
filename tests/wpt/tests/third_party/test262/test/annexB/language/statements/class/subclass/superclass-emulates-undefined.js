// Copyright (C) 2020 Alexey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-runtime-semantics-classdefinitionevaluation
description: >
  [[IsHTMLDDA]] object as superclass: `null` check uses strict equality.
  IsConstructor check is performed before "prototype" lookup.
info: |
  ClassDefinitionEvaluation

  [...]
  5. Else,
    [...]
    d. Let superclass be ? GetValue(superclassRef).
    e. If superclass is null, then
      [...]
    f. Else if IsConstructor(superclass) is false, throw a TypeError exception.
features: [class, IsHTMLDDA]
---*/

var superclass = $262.IsHTMLDDA;
var prototypeGets = 0;
Object.defineProperty(superclass, "prototype", {
  get: function() {
    prototypeGets += 1;
  },
});

assert.throws(TypeError, function() {
  class C extends superclass {}
});

assert.sameValue(prototypeGets, 0);
