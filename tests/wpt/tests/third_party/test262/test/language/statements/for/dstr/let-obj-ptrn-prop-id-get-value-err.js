// This file was procedurally generated from the following sources:
// - src/dstr-binding/obj-ptrn-prop-id-get-value-err.case
// - src/dstr-binding/error/for-let.template
/*---
description: Error thrown when accessing the corresponding property of the value object (for statement)
esid: sec-for-statement-runtime-semantics-labelledevaluation
features: [destructuring-binding]
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

    13.3.3.7 Runtime Semantics: KeyedBindingInitialization

    BindingElement : BindingPattern Initializeropt

    1. Let v be GetV(value, propertyName).
    2. ReturnIfAbrupt(v).
---*/
var initEvalCount = 0;
var poisonedProperty = Object.defineProperty({}, 'poisoned', {
  get: function() {
    throw new Test262Error();
  }
});

assert.throws(Test262Error, function() {
  for (let { poisoned: x = ++initEvalCount } = poisonedProperty; ; ) {
    return;
  }
});

assert.sameValue(initEvalCount, 0);
