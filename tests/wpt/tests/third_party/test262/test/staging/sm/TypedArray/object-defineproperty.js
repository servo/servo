// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
includes: [sm/non262-TypedArray-shell.js]
description: |
  pending
esid: pending
---*/
var obj = new Int32Array(2);
obj[0] = 100;

var throws = [
    // Disallow accessors
    {get: undefined},
    {set: undefined},
    {get: undefined, set: undefined},
    {get: function() {}},
    {set: function() {}},
    {get: function() {}, set: function() {}},

    {configurable: false},
    {enumerable: false},
    {writable: false},

    {configurable: false, writable: true},
    {enumerable: false, configurable: false},

    {configurable: false, value: 15}
];

for (var desc of throws) {
    assert.throws(TypeError, function() { Object.defineProperty(obj, 0, desc); });
    assert.throws(TypeError, function() { Object.defineProperties(obj, {0: desc}); });
}

Object.defineProperty(obj, 0, {});
Object.defineProperty(obj, 0, {configurable: true});
Object.defineProperty(obj, 0, {enumerable: true});
Object.defineProperty(obj, 0, {writable: true});

assert.sameValue(obj[0], 100);

Object.defineProperty(obj, 0, {configurable: true, value: 15});
assert.sameValue(obj[0], 15);
Object.defineProperty(obj, 0, {enumerable: true, value: 16});
assert.sameValue(obj[0], 16);
Object.defineProperty(obj, 0, {writable: true, value: 17});
assert.sameValue(obj[0], 17);
Object.defineProperty(obj, 0, {value: 18});
assert.sameValue(obj[0], 18);

var desc = Object.getOwnPropertyDescriptor(obj, 0);
assert.sameValue(desc.configurable, true);
assert.sameValue(desc.enumerable, true);
assert.sameValue(desc.writable, true);
assert.sameValue(desc.value, 18);
assert.sameValue('get' in desc, false);
assert.sameValue('set' in desc, false);

Object.defineProperties(obj, {0: {value: 20}, 1: {value: 42}});
assert.sameValue(obj[0], 20);
assert.sameValue(obj[1], 42);

anyTypedArrayConstructors.forEach(constructor => {
    var obj = new constructor(4);
    obj[0] = 100;
    obj[1] = 200;

    for (var v of [20, 300, -10, Math.pow(2, 32), -Math.pow(2, 32), NaN]) {
        Object.defineProperty(obj, 0, {value: v});
        obj[1] = v;
        assert.sameValue(obj[0], obj[1]);
    }
});

