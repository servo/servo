// This file was procedurally generated from the following sources:
// - src/async-functions/await-as-identifier-reference.case
// - src/async-functions/syntax/async-obj-method.template
/*---
description: await is a reserved keyword within generator function bodies and may not be used as an identifier reference. (Async method)
esid: prod-AsyncMethod
features: [async-functions]
flags: [generated]
negative:
  phase: parse
  type: SyntaxError
info: |
    Async Function Definitions

    AsyncMethod :
      async [no LineTerminator here] PropertyName ( UniqueFormalParameters ) { AsyncFunctionBody }


    IdentifierReference : Identifier

    It is a Syntax Error if this production has a [Await] parameter and
    StringValue of Identifier is "await".

---*/
$DONOTEVALUATE();


var obj = {
  async method() {
    void await;
  }
};
