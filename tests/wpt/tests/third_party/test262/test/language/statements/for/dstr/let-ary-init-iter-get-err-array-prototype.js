// This file was procedurally generated from the following sources:
// - src/dstr-binding/ary-init-iter-get-err-array-prototype.case
// - src/dstr-binding/error/for-let.template
/*---
description: Abrupt completion returned by GetIterator (for statement)
esid: sec-for-statement-runtime-semantics-labelledevaluation
features: [Symbol.iterator, destructuring-binding]
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
  for (let [x, y, z] = [1, 2, 3]; ; ) {
    return;
  }
});
