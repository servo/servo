// This file was procedurally generated from the following sources:
// - src/async-functions/returns-async-arrow.case
// - src/async-functions/evaluation/async-class-expr-static-method.template
/*---
description: Async function returns an async function. (Static async method as a ClassExpression element)
esid: prod-AsyncMethod
features: [async-functions]
flags: [generated, async]
info: |
    ClassElement :
      static MethodDefinition

    MethodDefinition :
      AsyncMethod

    Async Function Definitions

    AsyncMethod :
      async [no LineTerminator here] PropertyName ( UniqueFormalParameters ) { AsyncFunctionBody }

---*/
let count = 0;


var C = class {
  static async method(x) {
    return async () => x;
  }
};
// Stores a reference `asyncFn` for case evaluation
let asyncFn = C.method;

asyncFn(1).then(retFn => {
  count++;
  return retFn();
}).then(result => {
  assert.sameValue(result, 1);
  assert.sameValue(count, 1);
}).then($DONE, $DONE);
