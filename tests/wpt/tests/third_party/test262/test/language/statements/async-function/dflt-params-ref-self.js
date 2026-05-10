// This file was procedurally generated from the following sources:
// - src/function-forms/dflt-params-ref-self.case
// - src/function-forms/error/async-func-decl.template
/*---
description: Referencing a parameter from within its own initializer (async function declaration)
esid: sec-async-function-definitions
features: [default-parameters, async-functions]
flags: [generated, async]
info: |
    14.6 Async Function Definitions

    AsyncFunctionDeclaration :
      async function BindingIdentifier ( FormalParameters ) { AsyncFunctionBody }


    14.1.19 Runtime Semantics: IteratorBindingInitialization

    FormalsList : FormalsList , FormalParameter

    1. Let status be the result of performing IteratorBindingInitialization for
       FormalsList using iteratorRecord and environment as the arguments.
    2. ReturnIfAbrupt(status).
    3. Return the result of performing IteratorBindingInitialization for
       FormalParameter using iteratorRecord and environment as the arguments.

---*/
var x = 0;


var callCount = 0;
async function f(x = x) {
  
  callCount = callCount + 1;
}

f()
  .then(_ => {
    throw new Test262Error('function should not be resolved');
  }, error => assert.sameValue(error.constructor, ReferenceError))
  .then(() => {
    assert.sameValue(callCount, 0, 'function body is not evaluated');
  }, $DONE)
  .then($DONE, $DONE);
