// This file was procedurally generated from the following sources:
// - src/function-forms/reassign-fn-name-in-body.case
// - src/function-forms/expr-named/func-expr-named-strict-error.template
/*---
description: Reassignment of function name is silently ignored in non-strict mode code. (named function expression in strict mode code)
esid: sec-function-definitions-runtime-semantics-evaluation
flags: [generated, onlyStrict]
info: |
    FunctionExpression : function BindingIdentifier ( FormalParameters ) { FunctionBody }

---*/

// increment callCount in case "body"
let callCount = 0;
let ref = function BindingIdentifier() {
  callCount++;
  BindingIdentifier = 1;
  return BindingIdentifier;
};

assert.throws(TypeError, () => {
  ref();
});
assert.sameValue(callCount, 1, 'function invoked exactly once');
