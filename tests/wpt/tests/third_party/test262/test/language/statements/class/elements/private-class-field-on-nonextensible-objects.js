// Copyright (C) 2019 Caio Lima. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: It is not possible to add private fields on non-extensible objects
esid: sec-define-field
info: |
  1.1 PrivateFieldAdd ( O, P, value )
    1. If O.[[Extensible]] is false, throw a TypeError exception.
    ...

features:
  - class
  - class-fields-private
  - class-fields-public
  - nonextensible-applies-to-private
flags: [onlyStrict]
---*/

// Analogous to
// test/language/statements/class/subclass/private-class-field-on-nonextensible-return-override.js

class NonExtensibleBase {
  constructor(seal) {
    if (seal) Object.preventExtensions(this);
  }
}


// extend superclass with private instance data field
class ClassWithPrivateField extends NonExtensibleBase {
  #val;

  constructor(seal) {
    super(seal);
    this.#val = 42;
  }
  val() {
    return this.#val;
  }
}

const t = new ClassWithPrivateField(false);
// extensible objects can be extended
assert.sameValue(t.val(), 42);

// where superclass prevented extensions & subclass extended
assert.throws(TypeError, function () {
  new ClassWithPrivateField(true);
});


// extend superclass with private instance method
class ClassWithPrivateMethod extends NonExtensibleBase {
  constructor(seal) {
    super(seal);
  }
  // private methods are on the instance, so will fail
  #privateMethod() {
    return 42;
  };
  // public methods are on the prototype, so are ok.
  publicMethod() {
    return this.#privateMethod();
  }
}

const m = new ClassWithPrivateMethod(false);
// extensible objects can be extended
assert.sameValue(m.publicMethod(), 42);

// where superclass prevented extensions & subclass extended
assert.throws(TypeError, function () {
  new ClassWithPrivateMethod(true);
});


// extend superclass with private instance accessor
class ClassWithPrivateAccessor extends NonExtensibleBase {
  constructor(seal) {
    super(seal);
  }
  // private accessors are on the instance, so will fail
  get #privateAccessor() {
    return 42;
  };
  // public accessors are on the prototype, so are ok.
  get publicAccessor() {
    return this.#privateAccessor;
  }
}

const a = new ClassWithPrivateAccessor(false);
// extensible objects can be extended
assert.sameValue(a.publicAccessor, 42);

// where superclass prevented extensions & subclass extended
assert.throws(TypeError, function () {
  new ClassWithPrivateAccessor(true);
});


// base class private instance data field
class TestNonExtensibleData {
  #g = (Object.preventExtensions(this), "Test262");
}

assert.throws(TypeError, function () {
  new TestNonExtensibleData();
});

// base class with private static data field
assert.throws(TypeError, function () {
  class TestNonExtensibleStaticData {
    static #g = (Object.preventExtensions(TestNonExtensibleStaticData), "Test262");
  }
});
