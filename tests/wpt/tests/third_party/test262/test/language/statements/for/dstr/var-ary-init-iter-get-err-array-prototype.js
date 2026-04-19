// This file was procedurally generated from the following sources:
// - src/dstr-binding/ary-init-iter-get-err-array-prototype.case
// - src/dstr-binding/error/for-var.template
/*---
description: Abrupt completion returned by GetIterator (for statement)
esid: sec-for-statement-runtime-semantics-labelledevaluation
features: [Symbol.iterator, destructuring-binding]
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

    Runtime Semantics: BindingInitialization

    BindingPattern : ArrayBindingPattern

    1. Let iteratorRecord be ? GetIterator(value).

    GetIterator ( obj [ , hint [ , method ] ] )

    [...]
    4. Let iterator be ? Call(method, obj).

    Call ( F, V [ , argumentsList ] )

    [...]
    2. If IsCallable(F) is false, throw a TypeError exception.

---*/
delete Array.prototype[Symbol.iterator];

assert.throws(TypeError, function() {
  for (var [x, y, z] = [1, 2, 3]; iterCount < 1; ) {
    return;
  }
});
