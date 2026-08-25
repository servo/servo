// This file was procedurally generated from the following sources:
// - src/function-forms/reassign-fn-name-in-body-in-eval.case
// - src/function-forms/expr-named/gen-func-expr-named-no-strict.template
/*---
description: Reassignment of function name is silently ignored in non-strict mode code. (named generator function expression in non-strict mode code)
esid: sec-generator-function-definitions-runtime-semantics-evaluation
features: [generators]
flags: [generated, noStrict]
info: |
    GeneratorExpression : function * BindingIdentifier ( FormalParameters ) { GeneratorBody }

---*/

// increment callCount in case "body"
let callCount = 0;
let ref = function * BindingIdentifier() {
  callCount++;
  eval("BindingIdentifier = 1");
  return BindingIdentifier;
};

assert.sameValue(ref().next().value, ref);
assert.sameValue(callCount, 1, 'function invoked exactly once');
