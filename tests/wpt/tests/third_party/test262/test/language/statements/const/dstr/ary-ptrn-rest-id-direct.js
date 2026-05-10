// This file was procedurally generated from the following sources:
// - src/dstr-binding/ary-ptrn-rest-id-direct.case
// - src/dstr-binding/default/const-stmt.template
/*---
description: Lone rest element (direct binding) (`const` statement)
esid: sec-let-and-const-declarations-runtime-semantics-evaluation
features: [destructuring-binding]
flags: [generated]
includes: [compareArray.js]
info: |
    LexicalBinding : BindingPattern Initializer

    1. Let rhs be the result of evaluating Initializer.
    2. Let value be GetValue(rhs).
    3. ReturnIfAbrupt(value).
    4. Let env be the running execution context's LexicalEnvironment.
    5. Return the result of performing BindingInitialization for BindingPattern
       using value and env as the arguments.

    Runtime Semantics: IteratorBindingInitialization

    BindingRestElement : ... BindingIdentifier

    [...]
    2. Let A be ! ArrayCreate(0).
    3. Let n be 0.
    4. Repeat,
        [...]
        f. Perform ! CreateDataPropertyOrThrow(A, ! ToString(n), nextValue).
        g. Set n to n + 1.

---*/

const [...x] = [1];

assert(Array.isArray(x));
assert.compareArray(x, [1]);
