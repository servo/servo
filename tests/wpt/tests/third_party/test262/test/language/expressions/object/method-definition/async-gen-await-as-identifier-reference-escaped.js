// This file was procedurally generated from the following sources:
// - src/async-generators/await-as-identifier-reference-escaped.case
// - src/async-generators/syntax/async-obj-method.template
/*---
description: await is a reserved keyword within generator function bodies and may not be used as an identifier reference. (Async generator method)
esid: prod-AsyncGeneratorMethod
features: [async-iteration]
flags: [generated]
negative:
  phase: parse
  type: SyntaxError
info: |
    Async Generator Function Definitions

    AsyncGeneratorMethod :
      async [no LineTerminator here] * PropertyName ( UniqueFormalParameters ) { AsyncGeneratorBody }


    IdentifierReference : Identifier

    It is a Syntax Error if this production has a [Await] parameter and
    StringValue of Identifier is "await".

---*/
$DONOTEVALUATE();

var obj = {
  async *method() {
    void \u0061wait;
  }
};
