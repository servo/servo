// This file was procedurally generated from the following sources:
// - src/dstr-binding/ary-ptrn-elision-exhausted.case
// - src/dstr-binding/default/var-stmt.template
/*---
description: Elision accepts exhausted iterator (`var` statement)
esid: sec-variable-statement-runtime-semantics-evaluation
features: [generators, destructuring-binding]
flags: [generated]
info: |
    VariableDeclaration : BindingPattern Initializer

    1. Let rhs be the result of evaluating Initializer.
    2. Let rval be GetValue(rhs).
    3. ReturnIfAbrupt(rval).
    4. Return the result of performing BindingInitialization for
       BindingPattern passing rval and undefined as arguments.

    13.3.3.6 Runtime Semantics: IteratorBindingInitialization

    ArrayBindingPattern : [ Elision ]

    1. Return the result of performing
       IteratorDestructuringAssignmentEvaluation of Elision with iteratorRecord
       as the argument.

    12.14.5.3 Runtime Semantics: IteratorDestructuringAssignmentEvaluation

    Elision : ,

    1. If iteratorRecord.[[done]] is false, then
       [...]
    2. Return NormalCompletion(empty).

---*/
var iter = function*() {}();
iter.next();

var [,] = iter;


