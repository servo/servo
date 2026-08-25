// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  pending
esid: pending
---*/
class TestClass {
    constructor() { }
    method() { }
    get getter() { }
    set setter(x) { }
    *generator() { }
    static staticMethod() { }
    static get staticGetter() { }
    static set staticSetter(x) { }
    static *staticGenerator() { }
}

var test = new TestClass();

var hasPrototype = [
    test.constructor,
    test.generator,
    TestClass.staticGenerator
]

for (var fun of hasPrototype) {
    assert.sameValue(fun.hasOwnProperty('prototype'), true);
}

var hasNoPrototype = [
    test.method,
    Object.getOwnPropertyDescriptor(test.__proto__, 'getter').get,
    Object.getOwnPropertyDescriptor(test.__proto__, 'setter').set,
    TestClass.staticMethod,
    Object.getOwnPropertyDescriptor(TestClass, 'staticGetter').get,
    Object.getOwnPropertyDescriptor(TestClass, 'staticSetter').set,
]

for (var fun of hasNoPrototype) {
    assert.sameValue(fun.hasOwnProperty('prototype'), false);
}

