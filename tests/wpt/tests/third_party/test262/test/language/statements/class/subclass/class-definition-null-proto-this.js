// Copyright (C) 2014 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-runtime-semantics-classdefinitionevaluation
description: >
  The `this` value of a null-extending class isn't automatically initialized
info: |
  Runtime Semantics: ClassDefinitionEvaluation

  [...]
  15. If ClassHeritageopt is present, then set F's [[ConstructorKind]] internal slot to "derived".
  [...]

  12.2.2.1 Runtime Semantics: Evaluation
  PrimaryExpression : this
  1. Return ? ResolveThisBinding( ).

  8.3.4 ResolveThisBinding ( )
  [...]
  2. Return ? envRec.GetThisBinding().
  
  8.1.1.3.4 GetThisBinding ( )
  [...]
  3. If envRec.[[ThisBindingStatus]] is "uninitialized", throw a ReferenceError exception.
  [...]
---*/

class C extends null {
  constructor() {
    // Use an arrow function to access the `this` binding of the class constructor.
    assert.throws(ReferenceError, () => {
      this;
    });
  }
}

assert.throws(ReferenceError, function() {
  new C();
});
