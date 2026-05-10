// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: >
    Imported binding reflects state of exported default binding ("anonymous"
    class declaration)
esid: sec-moduledeclarationinstantiation
info: |
    [...]
    17. For each element d in lexDeclarations do
        a. For each element dn of the BoundNames of d do
           i. If IsConstantDeclaration of d is true, then
              [...]
           ii. Else,
               1. Perform ! envRec.CreateMutableBinding(dn, false).
           iii. If d is a GeneratorDeclaration production or a
                FunctionDeclaration production, then
                1. Let fo be the result of performing InstantiateFunctionObject
                   for d with argument env.
                2. Call envRec.InitializeBinding(dn, fo).
    [...]

    14.5 Class Definitions

    Syntax

    ClassDeclaration[Yield, Default]:

        class BindingIdentifier[?Yield] ClassTail[?Yield]
        [+Default] class ClassTail[?Yield]
flags: [module]
---*/

assert.throws(ReferenceError, function() {
  typeof C;
}, 'Binding is created but not initialized.');

export default class {};
import C from './instn-named-bndng-dflt-cls.js';
