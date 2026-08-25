// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-globaldeclarationinstantiation
es6id: 15.1.8
description: Var binding collision with existing lexical declaration
info: |
  [...]
  6. For each name in varNames, do
     a. If envRec.HasLexicalDeclaration(name) is true, throw a SyntaxError
        exception.
---*/

var test262Var;
let test262Let;
const test262Const = null;
class test262Class {}

$262.evalScript('var test262Var;');
$262.evalScript('function test262Var() {}');

assert.throws(SyntaxError, function() {
  $262.evalScript('var x; var test262Let;');
}, '`var` on `let` binding');
assert.throws(ReferenceError, function() {
  x;
}, 'no bindings created (script declaring a `var` on a `let` binding)');

assert.throws(SyntaxError, function() {
  $262.evalScript('var x; var test262Const;');
}, '`var` on `const` binding');
assert.throws(ReferenceError, function() {
  x;
}, 'no bindings created (script declaring a `var` on a `const` binding)');

assert.throws(SyntaxError, function() {
  $262.evalScript('var x; var test262Class;');
}, '`var` on `class` binding');
assert.throws(ReferenceError, function() {
  x;
}, 'no bindings created (script declaring a `var` on a `class` binding)');

assert.throws(SyntaxError, function() {
  $262.evalScript('var x; function test262Let() {}');
}, 'function on `let` binding');
assert.throws(ReferenceError, function() {
  x;
}, 'no bindings created (script declaring a function on a `let` binding)');

assert.throws(SyntaxError, function() {
  $262.evalScript('var x; function test262Const() {}');
}, 'function on `const` binding');
assert.throws(ReferenceError, function() {
  x;
}, 'no bindings created (script declaring a function on a `const` binding)');

assert.throws(SyntaxError, function() {
  $262.evalScript('var x; function test262Class() {}');
} , 'function on `class` binding');
assert.throws(ReferenceError, function() {
  x;
}, 'no bindings created (script declaring a function on a class binding)');
