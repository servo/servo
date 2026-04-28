// This file was procedurally generated from the following sources:
// - src/dstr-binding/ary-ptrn-elem-obj-val-undef.case
// - src/dstr-binding/error/var-stmt.template
/*---
description: Nested object destructuring with a value of `undefined` (`var` statement)
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

    BindingElement : BindingPattern Initializeropt

    1. If iteratorRecord.[[done]] is false, then
       [...]
       e. Else
          i. Let v be IteratorValue(next).
          [...]
    4. Return the result of performing BindingInitialization of BindingPattern
       with v and environment as the arguments.

    13.3.3.5 Runtime Semantics: BindingInitialization

    BindingPattern : ObjectBindingPattern

    1. Let valid be RequireObjectCoercible(value).
    2. ReturnIfAbrupt(valid).
---*/

assert.throws(TypeError, function() {
  var [{ x }] = [];
});
