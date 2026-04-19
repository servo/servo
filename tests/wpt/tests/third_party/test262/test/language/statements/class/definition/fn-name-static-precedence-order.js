// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-runtime-semantics-classdefinitionevaluation
description: >
    Function `name` attribute not inferred in presence of static `name` method
info: |
    ClassTail : ClassHeritage_opt { ClassBody_opt }

    14. If constructor is empty, then [...]
      b. Let F be ! CreateBuiltinFunction(steps, 0, className, « [[ConstructorKind]], [[SourceText]] », empty, constructorParent).
    15. Else, [...]
      d. Perform ! SetFunctionName(F, className).
    25. For each ClassElement e of elements, do
      a. If IsStatic of e is false, then [...]
      b. Else,
        i. Let field be ClassElementEvaluation of e with arguments F and false.
           [ This overwrites the name property on F. ]
includes: [compareArray.js]
features: [generators]
---*/

class A {
  static method() {
    throw new Test262Error('Static method should not be executed during definition');
  }
  static name() {
    throw new Test262Error('Static method should not be executed during definition');
  }
}

assert.compareArray(Object.getOwnPropertyNames(A), ['length', 'name', 'prototype', 'method'])

var attr = 'name';
class B {
  static [attr]() {
    throw new Test262Error(
      'Static method defined via computed property should not be executed ' +
      'during definition'
    );
  }
}

assert.compareArray(Object.getOwnPropertyNames(B), ['length', 'name', 'prototype'])

class C {
  static get name() {
    throw new Test262Error('Static `get` accessor should not be executed during definition');
  }
}

assert.compareArray(Object.getOwnPropertyNames(C), ['length', 'name', 'prototype'])

class D {
  static set name(_) {
    throw new Test262Error('Static `set` accessor should not be executed during definition');
  }
}

assert.compareArray(Object.getOwnPropertyNames(D), ['length', 'name', 'prototype'])

class E {
  static *name() {
    throw new Test262Error('Static GeneratorMethod should not be executed during definition');
  }
}

assert.compareArray(Object.getOwnPropertyNames(E), ['length', 'name', 'prototype'])
