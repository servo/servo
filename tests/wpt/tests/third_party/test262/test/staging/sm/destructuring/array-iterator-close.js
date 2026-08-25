// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
includes: [sm/assertThrowsValue.js]
description: |
  Tests that IteratorClose is called in array destructuring patterns.
esid: pending
---*/

function test() {
    var returnCalled = 0;
    var returnCalledExpected = 0;
    var iterable = {};

    // empty [] calls IteratorClose regardless of "done" on the result.
    iterable[Symbol.iterator] = function() {
        return {
            next() {
                return { done: true };
            },
            return() {
                returnCalled++;
                return {};
            }
        };
    };
    var [] = iterable;
    assert.sameValue(returnCalled, ++returnCalledExpected);

    iterable[Symbol.iterator] = function() {
        return {
            next() {
                return { done: false };
            },
            return() {
                returnCalled++;
                return {};
            }
        };
    };
    var [] = iterable;
    assert.sameValue(returnCalled, ++returnCalledExpected);

    // Non-empty destructuring calls IteratorClose if iterator is not done by
    // the end of destructuring.
    var [a,b] = iterable;
    assert.sameValue(returnCalled, ++returnCalledExpected);
    var [c,] = iterable;
    assert.sameValue(returnCalled, ++returnCalledExpected);

    // throw in lhs ref calls IteratorClose
    function throwlhs() {
        throw "in lhs";
    }
    assertThrowsValue(function() {
        0, [...{}[throwlhs()]] = iterable;
    }, "in lhs");
    assert.sameValue(returnCalled, ++returnCalledExpected);

    // throw in lhs ref calls IteratorClose with falsy "done".
    iterable[Symbol.iterator] = function() {
        return {
            next() {
                // "done" is undefined.
                return {};
            },
            return() {
                returnCalled++;
                return {};
            }
        };
    };
    assertThrowsValue(function() {
        0, [...{}[throwlhs()]] = iterable;
    }, "in lhs");
    assert.sameValue(returnCalled, ++returnCalledExpected);

    // throw in iter.next doesn't call IteratorClose
    iterable[Symbol.iterator] = function() {
        return {
            next() {
                throw "in next";
            },
            return() {
                returnCalled++;
                return {};
            }
        };
    };
    assertThrowsValue(function() {
        var [d] = iterable;
    }, "in next");
    assert.sameValue(returnCalled, returnCalledExpected);

    // "return" must return an Object.
    iterable[Symbol.iterator] = function() {
        return {
            next() {
                return { done: false };
            },
            return() {
                returnCalled++;
                return 42;
            }
        };
    };
    assert.throws(TypeError, function() {
        var [] = iterable;
    });
    assert.sameValue(returnCalled, ++returnCalledExpected);
}

test();

