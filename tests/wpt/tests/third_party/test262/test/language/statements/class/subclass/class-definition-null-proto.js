// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-runtime-semantics-classdefinitionevaluation
es6id: 14.5.14
description: >
  The prototype of a null-extending class is %FunctionPrototype%, the prototype of
  its "prototype" property is `null`.
info: |
  Runtime Semantics: ClassDefinitionEvaluation

  [...]
  5. If ClassHeritageopt is not present, then
     [...]
  6. Else,
     [...]
     b. Let superclass be the result of evaluating ClassHeritage.
     [...]
     e. If superclass is null, then
         i. Let protoParent be null.
        ii. Let constructorParent be the intrinsic object %FunctionPrototype%.
  [...]
---*/

class Foo extends null {}

assert.sameValue(Object.getPrototypeOf(Foo.prototype), null);
assert.sameValue(Object.getPrototypeOf(Foo.prototype.constructor), Function.prototype);
assert.sameValue(Foo, Foo.prototype.constructor);
