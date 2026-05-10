// This file was procedurally generated from the following sources:
// - src/dstr-binding/ary-ptrn-elem-ary-rest-iter.case
// - src/dstr-binding/default/var-stmt.template
/*---
description: BindingElement with array binding pattern and initializer is not used (`var` statement)
esid: sec-variable-statement-runtime-semantics-evaluation
features: [destructuring-binding]
flags: [generated]
info: |
    VariableDeclaration : BindingPattern Initializer

    1. Let rhs be the result of evaluating Initializer.
    2. Let rval be GetValue(rhs).
    3. ReturnIfAbrupt(rval).
    4. Return the result of performing BindingInitialization for
       BindingPattern passing rval and undefined as arguments.

    13.3.3.6 Runtime Semantics: IteratorBindingInitialization

    BindingElement : BindingPatternInitializer opt

    1. If iteratorRecord.[[done]] is false, then
       a. Let next be IteratorStep(iteratorRecord.[[iterator]]).
       [...]
       e. Else,
          i. Let v be IteratorValue(next).
          [...]
    4. Return the result of performing BindingInitialization of BindingPattern
       with v and environment as the arguments.
---*/
var values = [2, 1, 3];
var initCount = 0;

var [[...x] = function() { initCount += 1; }()] = [values];

assert(Array.isArray(x));
assert.sameValue(x[0], 2);
assert.sameValue(x[1], 1);
assert.sameValue(x[2], 3);
assert.sameValue(x.length, 3);
assert.notSameValue(x, values);
assert.sameValue(initCount, 0);
