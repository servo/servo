// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
flags:
  - noStrict
description: |
  pending
esid: pending
---*/
function testFunctionName(f) {
    var name = f.name;
    f.name = 'g';
    assert.sameValue(f.name, name);
    assert.sameValue(delete f.name, true);
    assert.sameValue(f.name, '');
    assert.sameValue(f.hasOwnProperty('name'), false);
    f.name = 'g';
    assert.sameValue(f.name, '');
    Object.defineProperty(f, 'name', {value: 'g'});
    assert.sameValue(f.name, 'g');
}
function testFunctionNameStrict(f) {
    "use strict";
    var name = f.name;
    assert.throws(TypeError, function() {
        f.name = 'g';
    });
    assert.sameValue(f.name, name);
    assert.sameValue(delete f.name, true);
    assert.sameValue(f.name, '');
    assert.sameValue(f.hasOwnProperty('name'), false);
    assert.throws(TypeError, function() {
        f.name = 'g';
    });
    assert.sameValue(f.name, '');
    Object.defineProperty(f, 'name', {value: 'g'});
    assert.sameValue(f.name, 'g');
}

assert.sameValue(Object.getOwnPropertyDescriptor(Object, "name").writable, false);
assert.sameValue(Object.getOwnPropertyDescriptor(Object, "name").enumerable, false);
assert.sameValue(Object.getOwnPropertyDescriptor(Object, "name").configurable, true);
assert.sameValue(Object.getOwnPropertyDescriptor(Object, "name").value, 'Object');
assert.sameValue(Object.getOwnPropertyDescriptor(function f(){}, "name").writable, false);
assert.sameValue(Object.getOwnPropertyDescriptor(function f(){}, "name").enumerable, false);
assert.sameValue(Object.getOwnPropertyDescriptor(function f(){}, "name").configurable, true);
assert.sameValue(Object.getOwnPropertyDescriptor(function f(){}, "name").value, 'f');

// Basic test ensuring that Object.defineProperty works on pristine function.
function f() {};
Object.defineProperty(f, 'name', {value: 'g'});
assert.sameValue(f.name, 'g');

// .name behaves as expected on scripted function.
testFunctionName(function f(){});
testFunctionNameStrict(function f(){});
// .name behaves as expected on builtin function.
testFunctionName(Function.prototype.apply);
testFunctionNameStrict(Function.prototype.call);
// .name behaves as expected on self-hosted builtin function.
testFunctionName(Array.prototype.forEach);
testFunctionNameStrict(Array.prototype.some);

