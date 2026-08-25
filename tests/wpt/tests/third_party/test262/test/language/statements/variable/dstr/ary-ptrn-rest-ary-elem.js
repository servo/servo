// This file was procedurally generated from the following sources:
// - src/dstr-binding/ary-ptrn-rest-ary-elem.case
// - src/dstr-binding/default/var-stmt.template
/*---
description: Rest element containing an array BindingElementList pattern (`var` statement)
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

    13.3.3.6 Runtime Semantics: IteratorBindingInitialization

    SingleNameBinding : BindingIdentifier Initializeropt

    [...]
    4. If iteratorRecord.[[done]] is false, then
       a. Let next be IteratorStep(iteratorRecord.[[iterator]]).
       b. If next is an abrupt completion, set iteratorRecord.[[done]] to true.
       c. ReturnIfAbrupt(next).
       d. If next is false, set iteratorRecord.[[done]] to true.
       e. Else,
          [...]
          i. Let v be IteratorValue(next).
          ii. If v is an abrupt completion, set
              iteratorRecord.[[done]] to true.
          iii. ReturnIfAbrupt(v).
    5. If iteratorRecord.[[done]] is true, let v be undefined.
    [...]
    8. Return InitializeReferencedBinding(lhs, v).
---*/

var [...[x, y, z]] = [3, 4, 5];

assert.sameValue(x, 3);
assert.sameValue(y, 4);
assert.sameValue(z, 5);
