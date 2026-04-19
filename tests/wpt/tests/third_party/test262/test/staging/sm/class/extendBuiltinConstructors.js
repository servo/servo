// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  pending
esid: pending
---*/
function testBuiltinInstanceIsInstanceOf(instance, builtin, class_) {
    assert.sameValue(instance instanceof class_, true);
    assert.sameValue(instance instanceof builtin, true);

    if (builtin === Array)
        assert.sameValue(Array.isArray(instance), true);
}

function testBuiltinInstance(builtin, ...args) {
    class sub extends builtin {
        constructor(...args) {
            super(...args);
            this.called = true;
        }
    }

    let instance = new sub(...args);
    assert.sameValue(instance.called, true);
    testBuiltinInstanceIsInstanceOf(instance, builtin, sub);
}

function testBuiltinMultipleSubclasses(builtin, ...args) {
    function f(obj, prop) {
        assert.sameValue(obj.prop, prop);
    }

    class sub1 extends builtin { };
    class sub2 extends builtin { };

    const prop1 = "A";
    const prop2 = "B";

    sub1.prototype.prop = prop1;
    sub2.prototype.prop = prop2;

    let instance1 = new sub1(...args);
    let instance2 = new sub2(...args);

    // Also make sure we get the properties we want with a default constructor
    testBuiltinInstanceIsInstanceOf(instance1, builtin, sub1);

    for (let i = 0; i < 10; i++) {
        f(instance1, prop1);
        f(instance2, prop2);
    }
}

function testBuiltin(builtin, ...args) {
    testBuiltinInstance(builtin, ...args);
    testBuiltinMultipleSubclasses(builtin, ...args);
}

function testBuiltinTypedArrays() {
    let typedArrays = [Int8Array,
                       Uint8Array,
                       Uint8ClampedArray,
                       Int16Array,
                       Uint16Array,
                       Int32Array,
                       Uint32Array,
                       Float32Array,
                       Float64Array];

    for (let array of typedArrays) {
        testBuiltin(array);
        testBuiltin(array, 5);
        testBuiltin(array, new array());
        testBuiltin(array, new ArrayBuffer());
    }
}

testBuiltin(Function);
testBuiltin(Object);
testBuiltin(Boolean);
testBuiltin(Error);
testBuiltin(EvalError);
testBuiltin(RangeError);
testBuiltin(ReferenceError);
testBuiltin(SyntaxError);
testBuiltin(TypeError);
testBuiltin(URIError);
testBuiltin(Number);
testBuiltin(Date);
testBuiltin(Date, 5);
testBuiltin(Date, 5, 10);
testBuiltin(RegExp);
testBuiltin(RegExp, /Regexp Argument/);
testBuiltin(RegExp, "String Argument");
testBuiltin(Map);
testBuiltin(Set);
testBuiltin(WeakMap);
testBuiltin(WeakSet);
testBuiltin(ArrayBuffer);
testBuiltinTypedArrays();
testBuiltin(DataView, new ArrayBuffer());
testBuiltin(DataView, new ($262.createRealm().global.ArrayBuffer)());
testBuiltin(String);
testBuiltin(Array);
testBuiltin(Array, 15);
testBuiltin(Array, 3.0);
testBuiltin(Array, "non-length one-arg");
testBuiltin(Array, 5, 10, 15, "these are elements");
// More Promise subclassing tests can be found in non262/Promise/promise-subclassing.js
testBuiltin(Promise, _=>{});

if (this.SharedArrayBuffer)
    testBuiltin(SharedArrayBuffer);

