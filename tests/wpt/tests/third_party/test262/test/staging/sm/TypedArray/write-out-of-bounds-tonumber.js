// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  pending
esid: pending
---*/
// ToNumber(value) is executed for OOB writes when using a direct assignment.
function plainSet() {
    var callCount = 0;
    var value = {
        valueOf() {
            callCount++;
            return 1;
        }
    };

    var N = 100;
    var ta = new Int32Array(0);
    for (var i = 0; i < N; ++i)
        ta[0] = value

    assert.sameValue(callCount, N);
}
for (var i = 0; i < 2; ++i) plainSet();

// ToNumber(value) is executed for OOB writes when using Reflect.set(...).
function reflectSet() {
    var callCount = 0;
    var value = {
        valueOf() {
            callCount++;
            return 1;
        }
    };

    var N = 100;
    var ta = new Int32Array(0);
    for (var i = 0; i < N; ++i)
        assert.sameValue(Reflect.set(ta, 0, value), true);

    assert.sameValue(callCount, N);
}
for (var i = 0; i < 2; ++i) reflectSet();

// ToNumber(value) is not executed for OOB writes when using Reflect.defineProperty(...).
function defineProp() {
    var callCount = 0;
    var value = {
        valueOf() {
            callCount++;
            return 1;
        }
    };
    var desc = {value, writable: true, enumerable: true, configurable: true};

    var N = 100;
    var ta = new Int32Array(0);
    for (var i = 0; i < N; ++i)
        assert.sameValue(Reflect.defineProperty(ta, 0, desc), false);

    assert.sameValue(callCount, 0);
}
for (var i = 0; i < 2; ++i) defineProp();

