// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: References to local `const` bindings resolve successfully
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

const constBinding = 89;
assert.sameValue(constBinding, 89);
