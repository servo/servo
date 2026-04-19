// This file was procedurally generated from the following sources:
// - src/dstr-binding/ary-ptrn-rest-id-elision-next-err.case
// - src/dstr-binding/error/for-const.template
/*---
description: Rest element following elision elements (for statement)
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
    ArrayBindingPattern : [ Elisionopt BindingRestElement ]
    1. If Elision is present, then
       a. Let status be the result of performing
          IteratorDestructuringAssignmentEvaluation of Elision with
          iteratorRecord as the argument.
       b. ReturnIfAbrupt(status).
    2. Return the result of performing IteratorBindingInitialization for
       BindingRestElement with iteratorRecord and environment as arguments.

---*/
var iter = (function*() { throw new Test262Error(); })();

assert.throws(Test262Error, function() {
  for (const [, ...x] = iter; ; ) {
    return;
  }
});
