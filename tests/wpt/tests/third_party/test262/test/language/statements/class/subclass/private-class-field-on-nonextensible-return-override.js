// Copyright (C) 2019 Caio Lima. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: It is not possible to add private fields on non-extensible objects via return override
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
// test/language/statements/class/elements/private-class-field-on-nonextensible-objects.js

class TrojanBase {
  constructor(obj) {
    return obj;
  }
}


// extend superclass with private instance data field
class ClassWithPrivateField extends TrojanBase {
  #val;

  constructor(obj) {
    super(obj);
    this.#val = 42;
  }
  static val(obj) {
    return obj.#val;
  }
}

const t = new ClassWithPrivateField({});
// extensible objects can be extended
assert.sameValue(ClassWithPrivateField.val(t), 42);

// where superclass prevented extensions & subclass extended
assert.throws(TypeError, function () {
  new ClassWithPrivateField(Object.preventExtensions({}));
});


// extend superclass with private instance method
class ClassWithPrivateMethod extends TrojanBase {
  constructor(obj) {
    super(obj);
  }
  // private methods are on the instance, so will fail
  #privateMethod() {
    return 42;
  };
  static val(obj) {
    return obj.#privateMethod();
  }
}

const m = new ClassWithPrivateMethod({});
// extensible objects can be extended
assert.sameValue(ClassWithPrivateMethod.val(m), 42);

// where superclass prevented extensions & subclass extended
assert.throws(TypeError, function () {
  new ClassWithPrivateMethod(Object.preventExtensions({}));
});


// extend superclass with private instance accessor
class ClassWithPrivateAccessor extends TrojanBase {
  constructor(obj) {
    super(obj);
  }
  // private accessors are on the instance, so will fail
  get #privateAccessor() {
    return 42;
  };
  static val(obj) {
    return obj.#privateAccessor;
  }
}

const a = new ClassWithPrivateAccessor({});
// extensible objects can be extended
assert.sameValue(ClassWithPrivateAccessor.val(a), 42);

// where superclass prevented extensions & subclass extended
assert.throws(TypeError, function () {
  new ClassWithPrivateAccessor(Object.preventExtensions({}));
});
