/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  pending
esid: pending
---*/
assert.sameValue(undefined, void 0);

assert.sameValue(Function.prototype.hasOwnProperty('prototype'), false);
assert.sameValue(Function.prototype.prototype, undefined);

var builtin_ctors = [
    Object, Function, Array, String, Boolean, Number, Date, RegExp, Error,
    EvalError, RangeError, ReferenceError, SyntaxError, TypeError, URIError
];

for (var i = 0; i < builtin_ctors.length; i++) {
    var c = builtin_ctors[i];
    assert.sameValue(typeof c.prototype, (c === Function) ? "function" : "object");
    assert.sameValue(c.prototype.constructor, c);
}

var builtin_funcs = [
    eval, isFinite, isNaN, parseFloat, parseInt,
    decodeURI, decodeURIComponent, encodeURI, encodeURIComponent
];

for (var i = 0; i < builtin_funcs.length; i++) {
    assert.sameValue(builtin_funcs[i].hasOwnProperty('prototype'), false);
    assert.sameValue(builtin_funcs[i].prototype, undefined);
}

var names = Object.getOwnPropertyNames(Math);
for (var i = 0; i < names.length; i++) {
    var m = Math[names[i]];
    if (typeof m === "function")
        assert.sameValue(m.prototype, undefined);
}

assert.sameValue(0, 0, "don't crash");
