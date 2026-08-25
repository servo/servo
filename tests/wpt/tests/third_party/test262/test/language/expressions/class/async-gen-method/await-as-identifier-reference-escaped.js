// This file was procedurally generated from the following sources:
// - src/async-generators/await-as-identifier-reference-escaped.case
// - src/async-generators/syntax/async-class-expr-method.template
/*---
description: await is a reserved keyword within generator function bodies and may not be used as an identifier reference. (Async generator method as a ClassExpression element)
esid: prod-AsyncGeneratorMethod
features: [async-iteration]
flags: [generated]
negative:
  phase: parse
  type: SyntaxError
info: |
    ClassElement :
      MethodDefinition

    MethodDefinition :
      AsyncGeneratorMethod

    Async Generator Function Definitions

    AsyncGeneratorMethod :
      async [no LineTerminator here] * PropertyName ( UniqueFormalParameters ) { AsyncGeneratorBody }


    IdentifierReference : Identifier

    It is a Syntax Error if this production has a [Await] parameter and
    StringValue of Identifier is "await".

---*/
$DONOTEVALUATE();


var C = class { async *gen() {
    void \u0061wait;
}};
