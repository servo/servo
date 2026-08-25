// This file was procedurally generated from the following sources:
// - src/dstr-binding/obj-ptrn-rest-getter.case
// - src/dstr-binding/default/for-let.template
/*---
description: Getter is called when obj is being deconstructed to a rest Object (for statement)
esid: sec-for-statement-runtime-semantics-labelledevaluation
features: [object-rest, destructuring-binding]
flags: [generated]
includes: [propertyHelper.js]
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
var count = 0;

var iterCount = 0;

for (let {...x} = { get v() { count++; return 2; } }; iterCount < 1; ) {
  assert.sameValue(count, 1);

  verifyProperty(x, "v", {
    enumerable: true,
    writable: true,
    configurable: true,
    value: 2
  });

  iterCount += 1;
}

assert.sameValue(iterCount, 1, 'Iteration occurred as expected');
