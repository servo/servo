// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
includes: [sm/assertThrowsValue.js]
description: |
  Computed Property Names
info: bugzilla.mozilla.org/show_bug.cgi?id=924688
esid: pending
---*/

// Function definitions.
function syntaxError (script) {
    assert.throws(SyntaxError, function() {
        Function(script);
    });
}


// Tests begin.

assert.throws(ReferenceError, function() { var a = {[field1]: "a", [field2]: "b"}; });

assert.throws(ReferenceError, function() {
                           field1 = 1;
                           var a = {[field1]: "a", [field2]: "b"};
                       });

var f1 = 1;
var f2 = 2;
var a = {[f1]: "a", [f2]: "b"};
assert.sameValue(a[1], "a");
assert.sameValue(a[2], "b");

var a = {[f1]: "a", f2: "b"};
assert.sameValue(a[1], "a");
assert.sameValue(a.f2, "b");

var a = {["f1"]: "a", [++f2]: "b"};
assert.sameValue(a.f1, "a");
assert.sameValue(a[3], "b");

var a = {["f1"]: "a", f2: "b"};
assert.sameValue(a.f1, "a");
assert.sameValue(a.f2, "b");


var i = 0;
var a = {
    ["foo" + ++i]: i,
    ["foo" + ++i]: i,
    ["foo" + ++i]: i
};
assert.sameValue(a.foo1, 1);
assert.sameValue(a.foo2, 2);
assert.sameValue(a.foo3, 3);

var expr = "abc";
syntaxError("({[");
syntaxError("({[expr");
syntaxError("({[expr]");
syntaxError("({[expr]})");
syntaxError("({[expr] 0})");
syntaxError("({[expr], 0})");
syntaxError("[[expr]: 0]");
syntaxError("({[expr]: name: 0})");
syntaxError("({[1, 2]: 3})");  // because '1,2' is an Expression but not an AssignmentExpression
syntaxError("({[1;]: 1})");    // and not an ExpressionStatement
syntaxError("({[if (0) 0;]})");  // much less a Statement
syntaxError("function f() { {[x]: 1} }");  // that's not even an ObjectLiteral
syntaxError("function f() { [x]: 1 }");    // or that
syntaxError('a = {[f1@]: "a", [f2]: "b"}'); // unexpected symbol at end of AssignmentExpression
assert.throws(SyntaxError, function() {
    JSON.parse('{["a"]:4}');
});

// Property characteristics.
a = { ["b"] : 4 };
var b = Object.getOwnPropertyDescriptor(a, "b");
assert.sameValue(b.configurable, true);
assert.sameValue(b.enumerable, true);
assert.sameValue(b.writable, true);
assert.sameValue(b.value, 4);

// Setter and getter are not hit.
Object.defineProperty(Object.prototype, "x", { set: function (x) { throw "FAIL"; },
                                               get: function (x) { throw "FAIL"; } });
var a = {["x"]: 0};
assert.sameValue(a.x, 0);

a = {["x"]: 1, ["x"]: 2};
assert.sameValue(a.x, 2);
a = {x: 1, ["x"]: 2};
assert.sameValue(a.x, 2);
a = {["x"]: 1, x: 2};
assert.sameValue(a.x, 2);

// Symbols
var unique_sym = Symbol("1"), registered_sym = Symbol.for("2");
a = { [unique_sym] : 2, [registered_sym] : 3 };
assert.sameValue(a[unique_sym], 2);
assert.sameValue(a[registered_sym], 3);

// Same expression can be run several times to build objects with different property names.
a = [];
for (var i = 0; i < 3; i++) {
    a[i] = {["foo" + i]: i};
}
assert.sameValue(a[0].foo0, 0);
assert.sameValue(a[1].foo1, 1);
assert.sameValue(a[2].foo2, 2);

// Following are stored in object's elements rather than slots.
var i = 0;
a = { [++i] : i,
      [++i] : i,
      [++i] : i,
      [++i] : i,
      [++i] : i,
      [++i] : i,
      [++i] : i,
      [++i] : i
}
for (var i = 1; i < 9; i++)
    assert.sameValue(a[i], i);
syntaxError("a.1");
syntaxError("a.2");
syntaxError("a.3");
syntaxError("a.4");
syntaxError("a.5");
syntaxError("a.6");
syntaxError("a.7");
syntaxError("a.8");

// Adding a single large index.
var i = 0;
a = { [++i] : i,
      [++i] : i,
      [++i] : i,
      [++i] : i,
      [++i] : i,
      [++i] : i,
      [++i] : i,
      [++i] : i,
      [3000] : 2999
}
for (var i = 1; i < 9; i++)
    assert.sameValue(a[i], i);
assert.sameValue(a[3000], 2999);

// Defining several properties using eval.
var code = "({";
for (i = 0; i < 1000; i++)
    code += "['foo' + " + i + "]: 'ok', "
code += "['bar']: 'ok'});";
var obj = eval(code);
for (i = 0; i < 1000; i++)
    assert.sameValue(obj["foo" + i], "ok");
assert.sameValue(obj["bar"], "ok");

// Can yield in a computed property name which is in a generator.
function* g() {
    var a = { [yield 1]: 2, [yield 2]: 3};
    return a;
}

var it = g();
var next = it.next();
assert.sameValue(next.done, false);
assert.sameValue(next.value, 1);
next = it.next("hello");
assert.sameValue(next.done, false);
assert.sameValue(next.value, 2);
next = it.next("world");
assert.sameValue(next.done, true);
assert.sameValue(next.value.hello, 2);
assert.sameValue(next.value.world, 3);

// get and set.
expr = "abc";
syntaxError("({get [");
syntaxError("({get [expr()");
syntaxError("({get [expr]()");
syntaxError("({get [expr]()})");
syntaxError("({get [expr] 0 ()})");
syntaxError("({get [expr], 0(})");
syntaxError("[get [expr]: 0()]");
syntaxError("({get [expr](: name: 0})");
syntaxError("({get [1, 2](): 3})");
syntaxError("({get [1;](): 1})");
syntaxError("({get [if (0) 0;](){}})");
syntaxError("({set [(a)");
syntaxError("({set [expr(a)");
syntaxError("({set [expr](a){}");
syntaxError("({set [expr]}(a)");
syntaxError("({set [expr](a), 0})");
syntaxError("[set [expr](a): 0]");
syntaxError("({set [expr](a): name: 0})");
syntaxError("({set [1, 2](a) {return 3;}})");
syntaxError("({set [1;](a) {return 1}})");
syntaxError("({set [if (0) 0;](a){}})");
syntaxError("function f() { {get [x](): 1} }");
syntaxError("function f() { get [x](): 1 }");
syntaxError("function f() { {set [x](a): 1} }");
syntaxError("function f() { set [x](a): 1 }");
f1 = "abc";
syntaxError('a = {get [f1@](){}, set [f1](a){}}'); // unexpected symbol at end of AssignmentExpression
syntaxError('a = {get@ [f1](){}, set [f1](a){}}'); // unexpected symbol after get
syntaxError('a = {get [f1](){}, set@ [f1](a){}}'); // unexpected symbol after set

expr = "hey";
a = {get [expr]() { return 3; }, set[expr](v) { throw 2; }};
assert.sameValue(a.hey, 3);
assertThrowsValue(() => { a.hey = 5; }, 2);

// Symbols with duplicate get and set.
expr = Symbol("hey");
a = {get [expr]() { return 3; }, set[expr](v) { throw 2; },
     set [expr] (w) { throw 4; }, get[expr](){return 5; }};
assert.sameValue(a[expr], 5);
assertThrowsValue(() => { a[expr] = 7; }, 4);

// expressions with side effects are called in the right order
var log = "";
obj = {
    "a": log += 'a',
    get [log += 'b']() {},
    [log += 'c']: log += 'd',
    set [log += 'e'](a) {}
};
assert.sameValue(log, "abcde");

// assignment expressions, objects and regex in computed names
obj = {
    get [a = "hey"]() { return 1; },
    get [a = {b : 4}.b]() { return 2; },
    set [/x/.source](a) { throw 3; }
}
assert.sameValue(obj.hey, 1);
assert.sameValue(obj[4], 2);
assertThrowsValue(() => { obj.x = 7; }, 3);
