// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
includes: [sm/non262-TypedArray-shell.js, deepEqual.js]
description: |
  pending
esid: pending
---*/
for (var constructor of anyTypedArrayConstructors) {
    // %TypedArray%.from works on arguments objects.
    (function () {
        assert.deepEqual(constructor.from(arguments), new constructor(["0", "1", undefined]));
    })("0", "1", undefined);

    // If an object has both .length and [@@iterator] properties, [@@iterator] is used.
    var a = ['0', '1', '2', '3', '4'];
    a[Symbol.iterator] = function* () {
        for (var i = 5; i--; )
            yield this[i];
    };

    var log = '';
    function f(x) {
        log += x;
        return x + x;
    }

    var b = constructor.from(a, f);
    assert.deepEqual(b, new constructor(['44', '33', '22', '11', '00']));
    assert.sameValue(log, '43210');

    // In fact, if [@@iterator] is present, .length isn't queried at all.
    var pa = new Proxy(a, {
        has: function (target, id) {
            if (id === "length")
                throw new Error(".length should not be queried (has)");
            return id in target;
        },
        get: function (target, id) {
            if (id === "length")
                throw new Error(".length should not be queried (get)");
            return target[id];
        },
        getOwnPropertyDescriptor: function (target, id) {
            if (id === "length")
                throw new Error(".length should not be queried (getOwnPropertyDescriptor)");
            return Object.getOwnPropertyDescriptor(target, id)
        }
    });
    log = "";
    b = constructor.from(pa, f);
    assert.deepEqual(b, new constructor(['44', '33', '22', '11', '00']));
    assert.sameValue(log, '43210');
}

