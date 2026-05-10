// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-globaldeclarationinstantiation
es6id: 15.1.8
description: Lexical declarations "shadow" configurable global properties
info: |
  [...]
  5. For each name in lexNames, do
     [...]
     c. Let hasRestrictedGlobal be ? envRec.HasRestrictedGlobalProperty(name).
     d. If hasRestrictedGlobal is true, throw a SyntaxError exception.
  [...]
  16. For each element d in lexDeclarations do
      a. NOTE Lexically declared names are only instantiated here but not
         initialized.
      b. For each element dn of the BoundNames of d do
         i. If IsConstantDeclaration of d is true, then
            1. Perform ? envRec.CreateImmutableBinding(dn, true).
         ii. Else,
             1. Perform ? envRec.CreateMutableBinding(dn, false).
---*/

let Array;

assert.sameValue(Array, undefined);
assert.sameValue(typeof this.Array, 'function');

// DO NOT USE propertyHelper API!
let descriptor = Object.getOwnPropertyDescriptor(this, 'Array');
assert.sameValue(descriptor.configurable, true);
assert.sameValue(descriptor.enumerable, false);
assert.sameValue(descriptor.writable, true);
