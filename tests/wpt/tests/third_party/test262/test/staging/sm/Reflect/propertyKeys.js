/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
includes: [sm/assertThrowsValue.js, compareArray.js, deepEqual.js]
description: |
  pending
esid: pending
---*/
// Test corner cases involving Reflect methods' propertyKey arguments.

// keys - Array of propertyKey values to be tested.
//
// Each element of this array is a record with these properties:
//
// *   value: a value that will be passed as a property key
//     to the various Reflect methods;
//
// *   expected: (optional) the string or symbol that ToPropertyKey(value)
//     should return. If this is omitted, ToPropertyKey(value) === value.
//
var keys = [
    {value: null, expected: "null"},
    {value: undefined, expected: "undefined"},
    {value: true, expected: "true"},
    {value: 42, expected: "42"},
    {value: "string"},
    {value: ""},
    {value: "string with \0"},
    {value: new String("ok"), expected: "ok"},
    {value: Symbol("sym")},
    {value: Symbol.iterator},
    {value: Object(Symbol.for("comet")), expected: Symbol.for("comet")},
    {
        value: {
            toString() { return "key"; },
            valueOf() { return "bad"; }
        },
        expected: "key"
    },
    {
        value: {
            toString: undefined,
            valueOf() { return "fallback"; }
        },
        expected: "fallback"
    },
    {
        value: {
            [Symbol.toPrimitive](hint) { return hint; }
        },
        expected: "string"
    },
    {
        value: {
            [Symbol.toPrimitive](hint) { return Symbol.for(hint); }
        },
        expected: Symbol.for("string")
    }
];

for (var {value, expected} of keys) {
    if (expected === undefined)
        expected = value;

    var obj = {};
    assert.sameValue(Reflect.defineProperty(obj, value, {value: 1, configurable: true}), true);
    assert.compareArray(Reflect.ownKeys(obj), [expected]);
    assert.deepEqual(Reflect.getOwnPropertyDescriptor(obj, value),
                 {value: 1,
                  writable: false,
                  enumerable: false,
                  configurable: true});
    assert.sameValue(Reflect.deleteProperty(obj, value), true);
    assert.sameValue(Reflect.has(obj, value), false);
    assert.sameValue(Reflect.set(obj, value, 113), true);
    assert.sameValue(obj[expected], 113);
    assert.sameValue(Reflect.has(obj, value), true);
    assert.sameValue(Reflect.get(obj, value), 113);
}

// ToPropertyKey can throw.
var exc = {};
var badKey = {toString() { throw exc; }};
var methodNames = ["defineProperty", "deleteProperty", "has", "get", "getOwnPropertyDescriptor", "set"];
for (var name of methodNames) {
    assertThrowsValue(() => Reflect[name]({}, badKey), exc);
}

