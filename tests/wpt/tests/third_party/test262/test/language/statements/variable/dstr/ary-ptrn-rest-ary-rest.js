// This file was procedurally generated from the following sources:
// - src/dstr-binding/ary-ptrn-rest-ary-rest.case
// - src/dstr-binding/default/var-stmt.template
/*---
description: Rest element containing a rest element (`var` statement)
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

    BindingRestElement : ... BindingPattern

    1. Let A be ArrayCreate(0).
    [...]
    3. Repeat
       [...]
       b. If iteratorRecord.[[done]] is true, then
          i. Return the result of performing BindingInitialization of
             BindingPattern with A and environment as the arguments.
       [...]
---*/
var values = [1, 2, 3];

var [...[...x]] = values;

assert(Array.isArray(x));
assert.sameValue(x.length, 3);
assert.sameValue(x[0], 1);
assert.sameValue(x[1], 2);
assert.sameValue(x[2], 3);
assert.notSameValue(x, values);

