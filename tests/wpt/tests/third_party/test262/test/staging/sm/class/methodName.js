// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  pending
esid: pending
---*/
class TestClass {
    get getter() { }
    set setter(x) { }
    method() { }

    static get staticGetter() { }
    static set staticSetter(x) { }
    static staticMethod() { }

    get 1() { }
    set 2(v) { }
    3() { }

    static get 4() { }
    static set 5(x) { }
    static 6() { }
}

function name(obj, property, get) {
    let desc = Object.getOwnPropertyDescriptor(obj, property);
    return (get ? desc.get : desc.set).name;
}

assert.sameValue(name(TestClass.prototype, "getter", true), "get getter");
assert.sameValue(name(TestClass.prototype, "setter", false), "set setter");
assert.sameValue(TestClass.prototype.method.name, "method");

assert.sameValue(name(TestClass, "staticGetter", true), "get staticGetter");
assert.sameValue(name(TestClass, "staticSetter", false), "set staticSetter");
assert.sameValue(TestClass.staticMethod.name, "staticMethod");

assert.sameValue(name(TestClass.prototype, "1", true), "get 1");
assert.sameValue(name(TestClass.prototype, "2", false), "set 2");
assert.sameValue(TestClass.prototype[3].name, "3");

assert.sameValue(name(TestClass, "4", true), "get 4");
assert.sameValue(name(TestClass, "5", false), "set 5");
assert.sameValue(TestClass[6].name, "6");

