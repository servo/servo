// This file was procedurally generated from the following sources:
// - src/dstr-binding/ary-ptrn-elision-iter-close.case
// - src/dstr-binding/iter-close/for-var.template
/*---
description: The iterator is properly consumed by the destructuring pattern (for statement)
esid: sec-for-statement-runtime-semantics-labelledevaluation
features: [generators, destructuring-binding]
flags: [generated]
info: |
    IterationStatement :
        for ( var VariableDeclarationList ; Expressionopt ; Expressionopt ) Statement

    1. Let varDcl be the result of evaluating VariableDeclarationList.
    [...]

    13.3.2.4 Runtime Semantics: Evaluation

    VariableDeclarationList : VariableDeclarationList , VariableDeclaration

    1. Let next be the result of evaluating VariableDeclarationList.
    2. ReturnIfAbrupt(next).
    3. Return the result of evaluating VariableDeclaration.

    VariableDeclaration : BindingPattern Initializer

    1. Let rhs be the result of evaluating Initializer.
    2. Let rval be GetValue(rhs).
    3. ReturnIfAbrupt(rval).
    4. Return the result of performing BindingInitialization for BindingPattern
       passing rval and undefined as arguments.
---*/
const iter = (function* () {
  yield;
  yield;
})();


function fn() {
  for (var [,] = iter; ; ) {
    return;
  }
}

fn();

assert.sameValue(iter.next().done, true, 'iteration occurred as expected');
