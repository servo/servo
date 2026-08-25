// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-globaldeclarationinstantiation
es6id: 15.1.8
description: Globally-declared lexical bindings cannot be deleted
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
flags: [noStrict]
---*/

$262.evalScript('let test262let;');

delete test262let;

// Binding values are asserted by a dedicated test. IdentifierReferences serve
// to ensure that the entries in the environment record persist.
test262let;

$262.evalScript('const test262const = null;');

delete test262const;

test262const;

$262.evalScript('class test262class {}');

delete test262class;

test262class;
