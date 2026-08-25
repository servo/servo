// This file was procedurally generated from the following sources:
// - src/async-functions/await-as-identifier-reference.case
// - src/async-functions/syntax/async-class-decl-private-method.template
/*---
description: await is a reserved keyword within generator function bodies and may not be used as an identifier reference. (Async private method as a ClassDeclaration element)
esid: prod-AsyncMethod
features: [async-functions, class-methods-private]
flags: [generated]
negative:
  phase: parse
  type: SyntaxError
info: |
    ClassElement :
      PrivateMethodDefinition

    MethodDefinition :
      AsyncMethod

    Async Function Definitions

    AsyncMethod :
      async [no LineTerminator here] # PropertyName ( UniqueFormalParameters ) { AsyncFunctionBody }


    IdentifierReference : Identifier

    It is a Syntax Error if this production has a [Await] parameter and
    StringValue of Identifier is "await".

---*/
$DONOTEVALUATE();


class C {
  async #method() {
    void await;
  }
}
