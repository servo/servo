// Copyright (C) 2020 Alexey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-properties-of-the-array-prototype-object
description: >
    Array.prototype methods resolve `this` value using strict mode semantics,
    throwing TypeError if called as top-level function.
info: |
    Array.prototype.concat ( ...items )

    1. Let O be ? ToObject(this value).

    ToObject ( argument )

    Argument Type: Undefined
    Result: Throw a TypeError exception.
features: [Symbol, Symbol.isConcatSpreadable, Symbol.iterator, Symbol.species, Array.prototype.flat, Array.prototype.flatMap, Array.prototype.includes]
---*/

["constructor", "length", "0", Symbol.isConcatSpreadable, Symbol.species].forEach(function(key) {
    Object.defineProperty(this, key, {
        get: function() {
            throw new Test262Error(String(key) + " lookup should not be performed");
        },
    });
}, this);

function callback() {
    throw new Test262Error("callback should not be called");
}

var index = {
    get valueOf() {
        throw new Test262Error("index should not be coerced to number");
    },
};

var separator = {
    get toString() {
        throw new Test262Error("separator should not be coerced to string");
    },
};

var concat = Array.prototype.concat;
assert.throws(TypeError, function() {
    concat();
}, "concat");

var copyWithin = Array.prototype.copyWithin;
assert.throws(TypeError, function() {
    copyWithin(index, index);
}, "copyWithin");

var entries = Array.prototype.entries;
assert.throws(TypeError, function() {
    entries();
}, "entries");

var every = Array.prototype.every;
assert.throws(TypeError, function() {
    every(callback);
}, "every");

var fill = Array.prototype.fill;
assert.throws(TypeError, function() {
    fill(0);
}, "fill");

var filter = Array.prototype.filter;
assert.throws(TypeError, function() {
    filter(callback);
}, "filter");

var find = Array.prototype.find;
assert.throws(TypeError, function() {
    find(callback);
}, "find");

var findIndex = Array.prototype.findIndex;
assert.throws(TypeError, function() {
    findIndex(callback);
}, "findIndex");

var flat = Array.prototype.flat;
assert.throws(TypeError, function() {
    flat(index);
}, "flat");

var flatMap = Array.prototype.flatMap;
assert.throws(TypeError, function() {
    flatMap(callback);
}, "flatMap");

var forEach = Array.prototype.forEach;
assert.throws(TypeError, function() {
    forEach(callback);
}, "forEach");

var includes = Array.prototype.includes;
assert.throws(TypeError, function() {
    includes(0, index);
}, "includes");

var indexOf = Array.prototype.indexOf;
assert.throws(TypeError, function() {
    indexOf(0, index);
}, "indexOf");

var join = Array.prototype.join;
assert.throws(TypeError, function() {
    join(separator);
}, "join");

var keys = Array.prototype.keys;
assert.throws(TypeError, function() {
    keys();
}, "keys");

var lastIndexOf = Array.prototype.lastIndexOf;
assert.throws(TypeError, function() {
    lastIndexOf(0, index);
}, "lastIndexOf");

var map = Array.prototype.map;
assert.throws(TypeError, function() {
    map(callback);
}, "map");

var pop = Array.prototype.pop;
assert.throws(TypeError, function() {
    pop();
}, "pop");

var push = Array.prototype.push;
assert.throws(TypeError, function() {
    push();
}, "push");

var reduce = Array.prototype.reduce;
assert.throws(TypeError, function() {
    reduce(callback, 0);
}, "reduce");

var reduceRight = Array.prototype.reduceRight;
assert.throws(TypeError, function() {
    reduceRight(callback, 0);
}, "reduceRight");

var reverse = Array.prototype.reverse;
assert.throws(TypeError, function() {
    reverse();
}, "reverse");

var shift = Array.prototype.shift;
assert.throws(TypeError, function() {
    shift();
}, "shift");

var slice = Array.prototype.slice;
assert.throws(TypeError, function() {
    slice(index, index);
}, "slice");

var some = Array.prototype.some;
assert.throws(TypeError, function() {
    some(callback);
}, "some");

var sort = Array.prototype.sort;
assert.throws(TypeError, function() {
    sort(callback);
}, "sort");

var splice = Array.prototype.splice;
assert.throws(TypeError, function() {
    splice(index, index);
}, "splice");

var toLocaleString = Array.prototype.toLocaleString;
assert.throws(TypeError, function() {
    toLocaleString();
}, "toLocaleString");

var toString = Array.prototype.toString;
assert.throws(TypeError, function() {
    toString();
}, "toString");

var unshift = Array.prototype.unshift;
assert.throws(TypeError, function() {
    unshift();
}, "unshift");

var values = Array.prototype.values;
assert.throws(TypeError, function() {
    values();
}, "values");

var iterator = Array.prototype[Symbol.iterator];
assert.throws(TypeError, function() {
    iterator();
}, "Symbol.iterator");
