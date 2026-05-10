// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
flags:
  - noStrict
description: |
  pending
esid: pending
---*/
// Test cases for arguments.length optimization.

function f1() {
    return arguments.length;
}

function f2(a, b, c) {
    return arguments.length;
}

// arrow functions don't have their own arguments, and so capture the enclosing
// scope.
function f3(a, b, c, d) {
    return (() => arguments.length)();
}

// Test a function which mutates arguments.length
function f4(a, b, c, d) {
    arguments.length = 42;
    return arguments.length;
}

// Manually read out arguments; should disable the length opt
function f5() {
    for (var i = 0; i < arguments.length; i++) {
        if (arguments[i] == 10) { return true }
    }
    return false;
}

function f6() {
    function inner() {
        return arguments.length;
    }
    return inner(1, 2, 3);
}

// edge cases of the arguments bindings:
function f7() {
    var arguments = 42;
    return arguments;
}

function f8() {
    var arguments = [1, 2];
    return arguments.length;
}

function f9() {
    eval("arguments.length = 42");
    return arguments.length;
}

function test() {
    assert.sameValue(f1(), 0);
    assert.sameValue(f1(1), 1);
    assert.sameValue(f1(1, 2), 2);
    assert.sameValue(f1(1, 2, 3), 3);

    assert.sameValue(f2(), 0);
    assert.sameValue(f2(1, 2, 3), 3);

    assert.sameValue(f3(), 0);
    assert.sameValue(f3(1, 2, 3), 3);

    assert.sameValue(f4(), 42);
    assert.sameValue(f4(1, 2, 3), 42);

    assert.sameValue(f5(), false);
    assert.sameValue(f5(1, 2, 3, 10), true);
    assert.sameValue(f5(1, 2, 3, 10, 20), true);
    assert.sameValue(f5(1, 2, 3, 9, 20, 30), false);

    assert.sameValue(f6(), 3)
    assert.sameValue(f6(1, 2, 3, 4), 3)

    assert.sameValue(f7(), 42);

    assert.sameValue(f8(), 2);

    assert.sameValue(f9(), 42);
}

test();

