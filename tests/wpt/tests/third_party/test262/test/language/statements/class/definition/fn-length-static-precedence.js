// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-runtime-semantics-classdefinitionevaluation
description: >
    Function `length` attribute not inferred in presence of static `length` method
info: |
    ClassTail : ClassHeritage_opt { ClassBody_opt }

    14. If constructor is empty, then [...]
      b. Let F be ! CreateBuiltinFunction(steps, 0, className, « [[ConstructorKind]], [[SourceText]] », empty, constructorParent).
    15. Else,
      a. Let constructorInfo be ! DefineMethod of constructor with arguments proto and constructorParent.
         [ This sets the length property on constructorInfo.[[Closure]]. ]
      b. Let F be constructorInfo.[[Closure]].
      [...]
    25. For each ClassElement e of elements, do
      a. If IsStatic of e is false, then [...]
      b. Else,
        i. Let field be ClassElementEvaluation of e with arguments F and false.
           [ This overwrites the length property on F. ]
features: [generators]
---*/

class A {
  static method() {
    throw new Test262Error('Static method should not be executed during definition');
  }
  static length() {
    throw new Test262Error('Static method should not be executed during definition');
  }
}

assert.sameValue(typeof A.length, 'function');

var attr = 'length';
class B {
  static [attr]() {
    throw new Test262Error(
      'Static method defined via computed property should not be executed ' +
      'during definition'
    );
  }
}

assert.sameValue(typeof B.length, 'function');

var isDefined = false;
class C {
  static get length() {
    if (isDefined) {
      return 'pass';
    }
    throw new Test262Error('Static `get` accessor should not be executed during definition');
  }
}

isDefined = true;
assert.sameValue(C.length, 'pass');

class D {
  static set length(_) {
    throw new Test262Error('Static `set` accessor should not be executed during definition');
  }
}

assert.sameValue(D.length, undefined);

class E {
  static *length() {
    throw new Test262Error('Static GeneratorMethod should not be executed during definition');
  }
}

assert.sameValue(typeof E.length, 'function');
