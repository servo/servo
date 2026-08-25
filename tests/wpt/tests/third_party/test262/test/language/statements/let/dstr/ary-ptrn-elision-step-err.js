// This file was procedurally generated from the following sources:
// - src/dstr-binding/ary-ptrn-elision-step-err.case
// - src/dstr-binding/error/let-stmt.template
/*---
description: Elision advances iterator and forwards abrupt completions (`let` statement)
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

    ArrayBindingPattern : [ Elision ]

    1. Return the result of performing
       IteratorDestructuringAssignmentEvaluation of Elision with iteratorRecord
       as the argument.

    12.14.5.3 Runtime Semantics: IteratorDestructuringAssignmentEvaluation

    Elision : ,

    1. If iteratorRecord.[[done]] is false, then
       a. Let next be IteratorStep(iteratorRecord.[[iterator]]).
       b. If next is an abrupt completion, set iteratorRecord.[[done]] to true.
       c. ReturnIfAbrupt(next).

---*/
var following = 0;
var iter =function* () {
  throw new Test262Error();
  following += 1;
}();

assert.throws(Test262Error, function() {
  let [,] = iter;
});

iter.next();
assert.sameValue(following, 0, 'Iterator was properly closed.');
