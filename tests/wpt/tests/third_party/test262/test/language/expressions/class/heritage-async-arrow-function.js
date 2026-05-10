// Copyright (C) 2020 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-runtime-semantics-classdefinitionevaluation
description: >
  Throw a TypeError exception if IsConstructor(superclass) is false (async arrow)
info: |
  Runtime Semantics: ClassDefinitionEvaluation

  ClassTail : ClassHeritage { ClassBody }
      ...

  5. Else,
      Set the running execution context's LexicalEnvironment to classScope.
      Let superclassRef be the result of evaluating ClassHeritage.
      Set the running execution context's LexicalEnvironment to env.
      Let superclass be ? GetValue(superclassRef).
      If superclass is null, then
          Let protoParent be null.
          Let constructorParent be %Function.prototype%.
      Else if IsConstructor(superclass) is false, throw a TypeError exception.
      ...
features: [class]
---*/


assert.throws(TypeError, () => {
  var C = class extends (async () => {}) {};
});

