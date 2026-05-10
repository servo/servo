// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: References to local `let` bindings resolve successfully
esid: sec-moduleevaluation
info: |
    8.1.1.5.1 GetBindingValue (N, S)

    [...]
    3. If the binding for N is an indirect binding, then
       [...]
    5. Return the value currently bound to N in envRec.

    13.3.1.4 Runtime Semantics: Evaluation

    LexicalBinding : BindingIdentifier Initializer

    [...]
    6. Return InitializeReferencedBinding(lhs, value).
flags: [module]
---*/

let letBinding = 1;
assert.sameValue(letBinding, 1);

letBinding = 2;
assert.sameValue(letBinding, 2);
