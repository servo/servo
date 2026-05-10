// This file was procedurally generated from the following sources:
// - src/function-forms/eval-var-scope-syntax-err.case
// - src/function-forms/error-no-strict/async-func-expr-named.template
/*---
description: sloppy direct eval in params introduces var (async function named expression in sloppy code)
esid: sec-async-function-definitions
features: [default-parameters, async-functions]
flags: [generated, async, noStrict]
info: |
    14.6 Async Function Definitions

    AsyncFunctionExpression :
      async function BindingIdentifier ( FormalParameters ) { AsyncFunctionBody }


    
    Runtime Semantics: IteratorBindingInitialization
    FormalParameter : BindingElement

    1. Return the result of performing IteratorBindingInitialization for BindingElement with arguments iteratorRecord and environment.

---*/


var callCount = 0;
var f = async function f(a = eval("var a = 42")) {
  
  callCount = callCount + 1;
}

f()
  .then(_ => {
    throw new Test262Error('function should not be resolved');
  }, error => assert.sameValue(error.constructor, SyntaxError))
  .then(() => {
    assert.sameValue(callCount, 0, 'function body is not evaluated');
  }, $DONE)
  .then($DONE, $DONE);
