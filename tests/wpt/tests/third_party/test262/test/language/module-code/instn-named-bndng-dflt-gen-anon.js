// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: >
    Imported binding reflects state of exported default binding ("anonymous"
    generator function declaration)
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

    14.4.12 Runtime Semantics: InstantiateFunctionObject

    GeneratorDeclaration : function * ( FormalParameters ) { GeneratorBody }

    1. If the function code for GeneratorDeclaration is strict mode code, let
       strict be true. Otherwise let strict be false.
    2. Let F be GeneratorFunctionCreate(Normal, FormalParameters,
       GeneratorBody, scope, strict).
    3. Let prototype be ObjectCreate(%GeneratorPrototype%).
    4. Perform DefinePropertyOrThrow(F, "prototype",
       PropertyDescriptor{[[Value]]: prototype, [[Writable]]: true,
       [[Enumerable]]: false, [[Configurable]]: false}).
    5. Perform SetFunctionName(F, "default").
    6. Return F.

    14.4 Generator Function Definitions

    Syntax

    GeneratorDeclaration[Yield, Default] :
         function * BindingIdentifier[?Yield] ( FormalParameters[Yield] ) { GeneratorBody }
         [+Default] function * ( FormalParameters[Yield] ) { GeneratorBody }
flags: [module]
features: [generators]
---*/

assert.sameValue(g().next().value, 23, 'generator function value is hoisted');
assert.sameValue(g.name, 'default', 'correct name is assigned');

import g from './instn-named-bndng-dflt-gen-anon.js';
export default function* () { return 23; };
