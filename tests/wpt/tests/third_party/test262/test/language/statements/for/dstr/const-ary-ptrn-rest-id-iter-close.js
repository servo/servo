// This file was procedurally generated from the following sources:
// - src/dstr-binding/ary-ptrn-rest-id-iter-close.case
// - src/dstr-binding/iter-close/for-const.template
/*---
description: The iterator is properly consumed by the destructuring pattern (for statement)
esid: sec-for-statement-runtime-semantics-labelledevaluation
features: [generators, destructuring-binding]
flags: [generated]
info: |
    IterationStatement :
        for ( LexicalDeclaration Expressionopt ; Expressionopt ) Statement

    [...]
    7. Let forDcl be the result of evaluating LexicalDeclaration.
    [...]

    LexicalDeclaration : LetOrConst BindingList ;

    1. Let next be the result of evaluating BindingList.
    2. ReturnIfAbrupt(next).
    3. Return NormalCompletion(empty).

    BindingList : BindingList , LexicalBinding

    1. Let next be the result of evaluating BindingList.
    2. ReturnIfAbrupt(next).
    3. Return the result of evaluating LexicalBinding.

    LexicalBinding : BindingPattern Initializer

    1. Let rhs be the result of evaluating Initializer.
    2. Let value be GetValue(rhs).
    3. ReturnIfAbrupt(value).
    4. Let env be the running execution context’s LexicalEnvironment.
    5. Return the result of performing BindingInitialization for BindingPattern
       using value and env as the arguments.
---*/
const iter = (function* () {
  yield;
  yield;
})();


function fn() {
  for (const [...x] = iter; ; ) {
    return;
  }
}

fn();

assert.sameValue(iter.next().done, true, 'iteration occurred as expected');
