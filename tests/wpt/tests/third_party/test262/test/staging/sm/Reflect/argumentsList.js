/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
includes: [sm/assertThrowsValue.js, sm/non262-Reflect-shell.js, compareArray.js]
description: |
  pending
esid: pending
---*/
// Tests for the argumentList argument to Reflect.apply and Reflect.construct.

// Reflect.apply and Reflect.construct require an argumentList argument that must be an object.
assert.throws(TypeError, () => Reflect.apply(Math.min, undefined));
assert.throws(TypeError, () => Reflect.construct(Object));
for (var primitive of SOME_PRIMITIVE_VALUES) {
    assert.throws(TypeError, () => Reflect.apply(Math.min, undefined, primitive));
    assert.throws(TypeError, () => Reflect.construct(Object, primitive));
}

// Array used by several tests below.
var BOTH = [
    Reflect.apply,
    // Adapt Reflect.construct to accept the same arguments as Reflect.apply.
    (target, thisArgument, argumentList) => Reflect.construct(target, argumentList)
];

// The argumentList is copied and becomes the list of arguments passed to the function.
function getRest(...x) { return x; }
var args = [1, 2, 3];
for (var method of BOTH) {
    var result = method(getRest, undefined, args);
    assert.sameValue(result.join(), args.join());
    assert.sameValue(result !== args, true);
}

// argumentList.length can be less than func.length.
function testLess(a, b, c, d, e) {
    assert.sameValue(a, 1);
    assert.sameValue(b, true);
    assert.sameValue(c, "three");
    assert.sameValue(d, Symbol.for);
    assert.sameValue(e, undefined);

    assert.sameValue(arguments.length, 4);
    assert.sameValue(arguments !== args, true);
    return "ok";
}
args = [1, true, "three", Symbol.for];
assert.sameValue(Reflect.apply(testLess, undefined, args), "ok");
assert.sameValue(Reflect.construct(testLess, args) instanceof testLess, true);

// argumentList.length can be more than func.length.
function testMoar(a) {
    assert.sameValue(a, args[0]);
    return "good";
}
assert.sameValue(Reflect.apply(testMoar, undefined, args), "good");
assert.sameValue(Reflect.construct(testMoar, args) instanceof testMoar, true);

// argumentList can be any object with a .length property.
function getArgs(...args) {
    return args;
}
for (var method of BOTH) {
    assert.compareArray(method(getArgs, undefined, {length: 0}), []);
    assert.compareArray(method(getArgs, undefined, {length: 1, "0": "zero"}), ["zero"]);
    assert.compareArray(method(getArgs, undefined, {length: 2}), [undefined, undefined]);
    assert.compareArray(method(getArgs, undefined, function (a, b, c) {}), [undefined, undefined, undefined]);
}

// The Iterable/Iterator interfaces are not used.
var funnyArgs = {
    0: "zero",
    1: "one",
    length: 2,
    [Symbol.iterator]() { throw "FAIL 1"; },
    next() { throw "FAIL 2"; }
};
for (var method of BOTH) {
    assert.compareArray(method(getArgs, undefined, funnyArgs), ["zero", "one"]);
}

// If argumentList has no .length property, no arguments are passed.
function count() { return {numArgsReceived: arguments.length}; }
for (var method of BOTH) {
    assert.sameValue(method(count, undefined, {"0": 0, "1": 1}).numArgsReceived,
             0);
    function* g() { yield 1; yield 2; }
    assert.sameValue(method(count, undefined, g()).numArgsReceived,
             0);
}

// If argumentsList.length has a getter, it is called.
var log;
args = {
    get length() { log += "L"; return 1; },
    get "0"() { log += "0"; return "zero"; },
    get "1"() { log += "1"; return "one"; }
};
for (var method of BOTH) {
    log = "";
    assert.compareArray(method(getArgs, undefined, args), ["zero"]);
    assert.sameValue(log, "L0");
}

// The argumentsList.length getter can throw; the exception is propagated.
var exc = {status: "bad"};
args = {
    get length() { throw exc; }
};
for (var method of BOTH) {
    assertThrowsValue(() => method(count, undefined, args), exc);
}

// argumentsList.length is converted to an integer.
for (var value of [1.7, "1", {valueOf() { return "1"; }}]) {
    args = {
        length: value,
        "0": "ponies"
    };
    for (var method of BOTH) {
        var result = method(getArgs, undefined, args);
        assert.sameValue(result.length, 1);
        assert.sameValue(result[0], "ponies");
    }
}

// If argumentsList.length is negative or NaN, no arguments are passed.
for (var method of BOTH) {
    for (var num of [-1, -0.1, -0, -1e99, -Infinity, NaN]) {
        assert.sameValue(method(count, undefined, {length: num}).numArgsReceived,
                 0);
    }
}

