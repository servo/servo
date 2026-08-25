// This file was procedurally generated from the following sources:
// - src/dstr-binding/ary-ptrn-rest-obj-prop-id.case
// - src/dstr-binding/default/let-stmt.template
/*---
description: Rest element containing an object binding pattern (`let` statement)
esid: sec-let-and-const-declarations-runtime-semantics-evaluation
features: [destructuring-binding]
flags: [generated]
info: |
    LexicalBinding : BindingPattern Initializer

    1. Let rhs be the result of evaluating Initializer.
    2. Let value be GetValue(rhs).
    3. ReturnIfAbrupt(value).
    4. Let env be the running execution context's LexicalEnvironment.
    5. Return the result of performing BindingInitialization for BindingPattern
       using value and env as the arguments.

    13.3.3.6 Runtime Semantics: IteratorBindingInitialization

    BindingRestElement : ... BindingPattern

    1. Let A be ArrayCreate(0).
    [...]
    3. Repeat
       [...]
       b. If iteratorRecord.[[done]] is true, then
          i. Return the result of performing BindingInitialization of
             BindingPattern with A and environment as the arguments.
       [...]
---*/
let length = "outer";

let [...{ 0: v, 1: w, 2: x, 3: y, length: z }] = [7, 8, 9];

assert.sameValue(v, 7);
assert.sameValue(w, 8);
assert.sameValue(x, 9);
assert.sameValue(y, undefined);
assert.sameValue(z, 3);

assert.sameValue(length, "outer", "the length prop is not set as a binding name");
