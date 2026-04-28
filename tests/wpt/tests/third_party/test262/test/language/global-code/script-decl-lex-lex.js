// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-globaldeclarationinstantiation
es6id: 15.1.8
description: Let binding collision with existing lexical declaration
info: |
  [...]
  5. For each name in lexNames, do
     a. If envRec.HasVarDeclaration(name) is true, throw a SyntaxError
        exception.
     b. If envRec.HasLexicalDeclaration(name) is true, throw a SyntaxError
        exception.
---*/

let test262Let;
const test262Const = null;
class test262Class {}

assert.throws(SyntaxError, function() {
  $262.evalScript('var x; let test262Let;');
}, '`let` binding');
assert.throws(ReferenceError, function() {
  x;
}, 'No bindings created for script containing `let` redeclaration');

assert.throws(SyntaxError, function() {
  $262.evalScript('var x; let test262Const;');
}, '`const` binding');
assert.throws(ReferenceError, function() {
  x;
}, 'No bindings created for script containing `const` redeclaration');

assert.throws(SyntaxError, function() {
  $262.evalScript('var x; let test262Class;');
}, '`class` binding');
assert.throws(ReferenceError, function() {
  x;
}, 'No bindings created for script containing `class` redeclaration');
