// This file was procedurally generated from the following sources:
// - src/dstr-binding/ary-ptrn-rest-ary-empty.case
// - src/dstr-binding/default/for-let.template
/*---
description: Rest element containing an "empty" array pattern (for statement)
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

    13.3.3.6 Runtime Semantics: IteratorBindingInitialization

    ArrayBindingPattern : [ ]

    1. Return NormalCompletion(empty).

---*/
var iterations = 0;
var iter = function*() {
  iterations += 1;
}();

var iterCount = 0;

for (let [...[]] = iter; iterCount < 1; ) {
  assert.sameValue(iterations, 1);

  iterCount += 1;
}

assert.sameValue(iterCount, 1, 'Iteration occurred as expected');
