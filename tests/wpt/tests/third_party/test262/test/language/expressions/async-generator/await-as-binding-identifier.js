// This file was procedurally generated from the following sources:
// - src/async-generators/await-as-binding-identifier.case
// - src/async-generators/syntax/async-expression.template
/*---
description: await is a reserved keyword within generator function bodies and may not be used as a binding identifier. (Unnamed async generator expression)
esid: prod-AsyncGeneratorExpression
features: [async-iteration]
flags: [generated]
negative:
  phase: parse
  type: SyntaxError
info: |
    Async Generator Function Definitions

    AsyncGeneratorExpression :
      async [no LineTerminator here] function * BindingIdentifier ( FormalParameters ) {
        AsyncGeneratorBody }


    BindingIdentifier : Identifier

    It is a Syntax Error if this production has a [Await] parameter and
    StringValue of Identifier is "await".

---*/
$DONOTEVALUATE();


var gen = async function *() {
  var await;
};
