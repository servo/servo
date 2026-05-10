// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
includes: [compareArray.js]
description: |
  pending
esid: pending
---*/

// This file was written by Andy Wingo <wingo@igalia.com> and originally
// contributed to V8 as generators-runtime.js, available here:
//
// https://source.chromium.org/chromium/chromium/src/+/main:v8/test/mjsunit/es6/generators-runtime.js

// Test aspects of the generator runtime.

// See http://people.mozilla.org/~jorendorff/es6-draft.html#sec-15.19.3.

function assertSyntaxError(str) {
    assert.throws(SyntaxError, Function(str));
}


function f() { "use strict"; }
function* g() { yield 1; }
var GeneratorFunctionPrototype = Object.getPrototypeOf(g);
var GeneratorFunction = GeneratorFunctionPrototype.constructor;
var GeneratorObjectPrototype = GeneratorFunctionPrototype.prototype;


// A generator function should have the same set of properties as any
// other function.
function TestGeneratorFunctionInstance() {
    var f_own_property_names = Object.getOwnPropertyNames(f);
    var g_own_property_names = Object.getOwnPropertyNames(g);

    f_own_property_names.sort();
    g_own_property_names.sort();

    assert.compareArray(f_own_property_names, g_own_property_names);
    var i;
    for (i = 0; i < f_own_property_names.length; i++) {
        var prop = f_own_property_names[i];
        var f_desc = Object.getOwnPropertyDescriptor(f, prop);
        var g_desc = Object.getOwnPropertyDescriptor(g, prop);
        assert.sameValue(f_desc.configurable, g_desc.configurable, prop);
        assert.sameValue(f_desc.writable, g_desc.writable, prop);
        assert.sameValue(f_desc.enumerable, g_desc.enumerable, prop);
    }
}
TestGeneratorFunctionInstance();


// Generators have an additional object interposed in the chain between
// themselves and Function.prototype.
function TestGeneratorFunctionPrototype() {
    // Sanity check.
    assert.sameValue(Object.getPrototypeOf(f), Function.prototype);
    assert.notSameValue(GeneratorFunctionPrototype, Function.prototype);
    assert.sameValue(Object.getPrototypeOf(GeneratorFunctionPrototype),
               Function.prototype);
    assert.sameValue(Object.getPrototypeOf(function* () {}),
               GeneratorFunctionPrototype);
}
TestGeneratorFunctionPrototype();


// Functions that we associate with generator objects are actually defined by
// a common prototype.
function TestGeneratorObjectPrototype() {
    // %GeneratorPrototype% must inherit from %IteratorPrototype%.
    var iterProto = Object.getPrototypeOf(Object.getPrototypeOf([][Symbol.iterator]()));
    assert.sameValue(Object.getPrototypeOf(GeneratorObjectPrototype),
             iterProto);
    assert.sameValue(Object.getPrototypeOf((function*(){yield 1}).prototype),
             GeneratorObjectPrototype);

    var expected_property_names = ["next", "return", "throw", "constructor"];
    var found_property_names =
        Object.getOwnPropertyNames(GeneratorObjectPrototype);

    expected_property_names.sort();
    found_property_names.sort();

    assert.compareArray(found_property_names, expected_property_names);
    assert.compareArray(Object.getOwnPropertySymbols(GeneratorObjectPrototype), [Symbol.toStringTag]);
}
TestGeneratorObjectPrototype();


// This tests the object that would be called "GeneratorFunction", if it were
// like "Function".
function TestGeneratorFunction() {
    assert.sameValue(GeneratorFunctionPrototype, GeneratorFunction.prototype);
    assert.sameValue(g instanceof GeneratorFunction, true);

    assert.sameValue(Function, Object.getPrototypeOf(GeneratorFunction));
    assert.sameValue(g instanceof Function, true);

    assert.sameValue("function* g() { yield 1; }", g.toString());

    // Not all functions are generators.
    assert.sameValue(f instanceof Function, true);  // Sanity check.
    assert.sameValue(f instanceof GeneratorFunction, false);

    assert.sameValue((new GeneratorFunction()) instanceof GeneratorFunction, true);
    assert.sameValue(GeneratorFunction() instanceof GeneratorFunction, true);

    assert.sameValue(GeneratorFunction('yield 1') instanceof GeneratorFunction, true);
    assert.sameValue(GeneratorFunction('return 1') instanceof GeneratorFunction, true);
    assert.sameValue(GeneratorFunction('a', 'yield a') instanceof GeneratorFunction, true);
    assert.sameValue(GeneratorFunction('a', 'return a') instanceof GeneratorFunction, true);
    assert.sameValue(GeneratorFunction('a', 'return a') instanceof GeneratorFunction, true);
    assertSyntaxError("GeneratorFunction('yield', 'return yield')");
    assert.sameValue(GeneratorFunction('with (x) return foo;') instanceof GeneratorFunction, true);
    assertSyntaxError("GeneratorFunction('\"use strict\"; with (x) return foo;')");

    // Doesn't matter particularly what string gets serialized, as long
    // as it contains "function*" and "yield 10".
    assert.sameValue(GeneratorFunction('yield 10').toString(),
             "function* anonymous(\n) {\nyield 10\n}");
}
TestGeneratorFunction();


function TestPerGeneratorPrototype() {
    assert.notSameValue((function*(){}).prototype, (function*(){}).prototype);
    assert.notSameValue((function*(){}).prototype, g.prototype);
    assert.sameValue(typeof GeneratorFunctionPrototype, "object");
    assert.sameValue(g.prototype.__proto__.constructor, GeneratorFunctionPrototype, "object");
    assert.sameValue(Object.getPrototypeOf(g.prototype), GeneratorObjectPrototype);
    assert.sameValue(g.prototype instanceof Function, false);
    assert.sameValue(typeof (g.prototype), "object");

    assert.compareArray(Object.getOwnPropertyNames(g.prototype), []);
}
TestPerGeneratorPrototype();
