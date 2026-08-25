// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: >
    Module Environment Records provide a this binding, and the value is
    `undefined`.
esid: sec-moduleevaluation
info: |
    [...]
    16. Let result be the result of evaluating module.[[ECMAScriptCode]].
    [...]

    12.2.2 The this Keyword
    12.2.2.1 Runtime Semantics: Evaluation

    PrimaryExpression : this

    1. Return ? ResolveThisBinding( ).

    8.3.4 ResolveThisBinding ( )

    1. Let envRec be GetThisEnvironment( ).
    2. Return ? envRec.GetThisBinding().

    8.3.3 GetThisEnvironment ( )

    1. Let lex be the running execution context's LexicalEnvironment.
    2. Repeat
       a. Let envRec be lex's EnvironmentRecord.
       b. Let exists be envRec.HasThisBinding().
       c. If exists is true, return envRec.
       d. Let outer be the value of lex's outer environment reference.
       e. Let lex be outer.

    8.1.1.5.3 HasThisBinding ()

    1. Return true.

    8.1.1.5.4 GetThisBinding ()

    1. Return undefined.
flags: [module]
---*/

assert.sameValue(this, undefined);
