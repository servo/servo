// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
includes: [deepEqual.js]
description: |
  pending
esid: pending
---*/
// Do the things we write in classes actually appear as they are supposed to?

var methodCalled;
var getterCalled;
var setterCalled;
var constructorCalled;
var staticMethodCalled;
var staticGetterCalled;
var staticSetterCalled;
class testClass {
    constructor() { constructorCalled = true; }
    __proto__() { methodCalled = true }
    get getter() { getterCalled = true; }
    set setter(x) { setterCalled = true; }
    static staticMethod() { staticMethodCalled = true; }
    static get staticGetter() { staticGetterCalled = true; }
    static set staticSetter(x) { staticSetterCalled = true; }
    *[Symbol.iterator]() { yield "cow"; yield "pig"; }
}

for (let a of [testClass,
                class {
                    constructor() { constructorCalled = true; }
                    __proto__() { methodCalled = true }
                    get getter() { getterCalled = true; }
                    set setter(x) { setterCalled = true; }
                    static staticMethod() { staticMethodCalled = true; }
                    static get staticGetter() { staticGetterCalled = true; }
                    static set staticSetter(x) { staticSetterCalled = true; }
                    *[Symbol.iterator]() { yield "cow"; yield "pig"; }
                }]) {

    methodCalled = false;
    getterCalled = false;
    setterCalled = false;
    constructorCalled = false;
    staticMethodCalled = false;
    staticGetterCalled = false;
    staticSetterCalled = false;

    var aConstDesc = Object.getOwnPropertyDescriptor(a.prototype, "constructor");
    assert.sameValue(aConstDesc.writable, true);
    assert.sameValue(aConstDesc.configurable, true);
    assert.sameValue(aConstDesc.enumerable, false);
    new aConstDesc.value();
    assert.sameValue(constructorCalled, true);

    // __proto__ is just an identifier for classes. No prototype changes are made.
    assert.sameValue(Object.getPrototypeOf(a.prototype), Object.prototype);
    var aMethDesc = Object.getOwnPropertyDescriptor(a.prototype, "__proto__");
    assert.sameValue(aMethDesc.writable, true);
    assert.sameValue(aMethDesc.configurable, true);
    assert.sameValue(aMethDesc.enumerable, false);
    aMethDesc.value();
    assert.sameValue(methodCalled, true);

    var aGetDesc = Object.getOwnPropertyDescriptor(a.prototype, "getter");
    assert.sameValue(aGetDesc.configurable, true);
    assert.sameValue(aGetDesc.enumerable, false);
    aGetDesc.get();
    assert.throws(TypeError, () => new aGetDesc.get);
    assert.sameValue(getterCalled, true);

    var aSetDesc = Object.getOwnPropertyDescriptor(a.prototype, "setter");
    assert.sameValue(aSetDesc.configurable, true);
    assert.sameValue(aSetDesc.enumerable, false);
    aSetDesc.set();
    assert.throws(TypeError, () => new aSetDesc.set);
    assert.sameValue(setterCalled, true);
    assert.deepEqual(aSetDesc, Object.getOwnPropertyDescriptor(a.prototype, "setter"));

    assert.sameValue(Object.getOwnPropertyDescriptor(new a(), "staticMethod"), undefined);
    var aStaticMethDesc = Object.getOwnPropertyDescriptor(a, "staticMethod");
    assert.sameValue(aStaticMethDesc.configurable, true);
    assert.sameValue(aStaticMethDesc.enumerable, false);
    assert.sameValue(aStaticMethDesc.writable, true);
    aStaticMethDesc.value();
    assert.throws(TypeError, () => new aStaticMethDesc.value);
    assert.sameValue(staticMethodCalled, true);

    assert.sameValue(Object.getOwnPropertyDescriptor(new a(), "staticGetter"), undefined);
    var aStaticGetDesc = Object.getOwnPropertyDescriptor(a, "staticGetter");
    assert.sameValue(aStaticGetDesc.configurable, true);
    assert.sameValue(aStaticGetDesc.enumerable, false);
    aStaticGetDesc.get();
    assert.throws(TypeError, () => new aStaticGetDesc.get);
    assert.sameValue(staticGetterCalled, true);

    assert.sameValue(Object.getOwnPropertyDescriptor(new a(), "staticSetter"), undefined);
    var aStaticSetDesc = Object.getOwnPropertyDescriptor(a, "staticSetter");
    assert.sameValue(aStaticSetDesc.configurable, true);
    assert.sameValue(aStaticSetDesc.enumerable, false);
    aStaticSetDesc.set();
    assert.throws(TypeError, () => new aStaticSetDesc.set);
    assert.sameValue(staticSetterCalled, true);

    assert.sameValue([...new a()].join(), "cow,pig");
}

