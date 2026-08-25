// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: >
    Imported binding reflects state of exported default binding ("anonymous"
    function declaration)
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

    14.1.20 Runtime Semantics: InstantiateFunctionObject

    FunctionDeclaration : function ( FormalParameters ) { FunctionBody }

    1. If the function code for FunctionDeclaration is strict mode code, let
       strict be true. Otherwise let strict be false.
    2. Let F be FunctionCreate(Normal, FormalParameters, FunctionBody, scope,
       strict).
    3. Perform MakeConstructor(F).
    4. Perform SetFunctionName(F, "default").
    5. Return F.

    14.1 Function Definitions

    Syntax

    FunctionDeclaration[Yield, Default] :

        function BindingIdentifier[?Yield] ( FormalParameters ) { FunctionBody }
        [+Default] function ( FormalParameters ) { FunctionBody }
flags: [module]
---*/

assert.sameValue(f(), 23, 'function value is hoisted');
assert.sameValue(f.name, 'default', 'correct name is assigned');

import f from './instn-named-bndng-dflt-fun-anon.js';
export default function() { return 23; };
