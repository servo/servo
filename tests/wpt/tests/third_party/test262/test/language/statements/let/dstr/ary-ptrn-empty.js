// This file was procedurally generated from the following sources:
// - src/dstr-binding/ary-ptrn-empty.case
// - src/dstr-binding/default/let-stmt.template
/*---
description: No iteration occurs for an "empty" array binding pattern (`let` statement)
esid: sec-let-and-const-declarations-runtime-semantics-evaluation
features: [generators, destructuring-binding]
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

    ArrayBindingPattern : [ ]

    1. Return NormalCompletion(empty).

---*/
var iterations = 0;
var iter = function*() {
  iterations += 1;
}();

let [] = iter;

assert.sameValue(iterations, 0);
