// Copyright (C) 2014 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-runtime-semantics-classdefinitionevaluation
description: >
  The `this` value of a null-extending class isn't automatically initialized,
  which makes it necessary to have an explicit return value in the constructor.
info: |
  Runtime Semantics: ClassDefinitionEvaluation

  [...]
  5. If ClassHeritageopt is not present, then
     [...]
  6. Else,
     [...]
     b. Let superclass be the result of evaluating ClassHeritage.
  [...]
  15. If ClassHeritageopt is present, then set F's [[ConstructorKind]] internal slot to "derived".
  [...]

  9.2.2 [[Construct]]

  [...]
  15. Return ? envRec.GetThisBinding().

  8.1.1.3.4 GetThisBinding ( )
  [...]
  3. If envRec.[[ThisBindingStatus]] is "uninitialized", throw a ReferenceError exception.
  [...]
---*/

class Foo extends null {
  constructor() {
  }
}

assert.throws(ReferenceError, function() {
  new C();
});
