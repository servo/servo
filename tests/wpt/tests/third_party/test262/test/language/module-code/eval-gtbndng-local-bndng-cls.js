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

    14.5.16 Runtime Semantics: Evaluation

    ClassDeclaration : class BindingIdentifier ClassTail

    1. Let status be the result of BindingClassDeclarationEvaluation of this
       ClassDeclaration.
    2. ReturnIfAbrupt(status).
    3. Return NormalCompletion(empty).

    14.5.15 Runtime Semantics: BindingClassDeclarationEvaluation

    [...]
    7. Perform ? InitializeBoundName(className, value, env).
    [...]
flags: [module]
---*/

class classBinding { valueOf() { return 33; } }
assert.sameValue(new classBinding().valueOf(), 33);

classBinding = 44;
assert.sameValue(classBinding, 44);
