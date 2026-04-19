// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
includes: [sm/assertThrowsValue.js]
description: |
  pending
esid: pending
---*/
function test() {
    var returnCalled = 0;
    var returnCalledExpected = 0;
    var catchEntered = 0;
    var finallyEntered = 0;
    var finallyEnteredExpected = 0;
    var iterable = {};
    iterable[Symbol.iterator] = function() {
        return {
            next() {
                return { done: false };
            },
            return() {
                returnCalled++;
                throw 42;
            }
        };
    };

    // inner try cannot catch IteratorClose throwing
    assertThrowsValue(function() {
        for (var x of iterable) {
            try {
                return;
            } catch (e) {
                catchEntered++;
            } finally {
                finallyEntered++;
            }
        }
    }, 42);
    assert.sameValue(returnCalled, ++returnCalledExpected);
    assert.sameValue(catchEntered, 0);
    assert.sameValue(finallyEntered, ++finallyEnteredExpected);
}

test();

