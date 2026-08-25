// This file was procedurally generated from the following sources:
// - src/dstr-binding/ary-ptrn-elem-obj-id-init.case
// - src/dstr-binding/default/var-stmt.template
/*---
description: BindingElement with object binding pattern and initializer is used (`var` statement)
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

    [...]
    2. If iteratorRecord.[[done]] is true, let v be undefined.
    3. If Initializer is present and v is undefined, then
       a. Let defaultValue be the result of evaluating Initializer.
       b. Let v be ? GetValue(defaultValue).
    4. Return the result of performing BindingInitialization of BindingPattern
       with v and environment as the arguments.
---*/

var [{ x, y, z } = { x: 44, y: 55, z: 66 }] = [];

assert.sameValue(x, 44);
assert.sameValue(y, 55);
assert.sameValue(z, 66);
