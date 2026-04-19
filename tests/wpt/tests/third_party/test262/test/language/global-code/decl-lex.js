// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-globaldeclarationinstantiation
es6id: 15.1.8
description: Declaration of lexical bindings
info: |
  [...]
  16. For each element d in lexDeclarations do
      a. NOTE Lexically declared names are only instantiated here but not
         initialized.
      b. For each element dn of the BoundNames of d do
         i. If IsConstantDeclaration of d is true, then
            1. Perform ? envRec.CreateImmutableBinding(dn, true).
         ii. Else,
             1. Perform ? envRec.CreateMutableBinding(dn, false).
  [...]
---*/

let test262let = 1;

test262let = 2;

assert.sameValue(test262let, 2, '`let` binding is mutable');
assert.sameValue(
  this.hasOwnProperty('test262let'),
  false,
  'property not created on the global object (let)'
);

const test262const = 3;

assert.throws(TypeError, function() {
  test262const = 4;
}, '`const` binding is strictly immutable');
assert.sameValue(test262const, 3, '`const` binding cannot be modified');
assert.sameValue(
  this.hasOwnProperty('test262const'),
  false,
  'property not created on the global object (const)'
);

class test262class {}

test262class = 5;

assert.sameValue(test262class, 5, '`class` binding is mutable');
assert.sameValue(
  this.hasOwnProperty('test262class'),
  false,
  'property not created on the global object (class)'
);
