// This file was procedurally generated from the following sources:
// - src/async-generators/yield-as-identifier-reference.case
// - src/async-generators/syntax/async-class-decl-private-method.template
/*---
description: yield is a reserved keyword within generator function bodies and may not be used as an identifier reference. (Async Generator private method as a ClassDeclaration element)
esid: prod-AsyncGeneratorMethod
features: [async-iteration, class-methods-private]
flags: [generated]
negative:
  phase: parse
  type: SyntaxError
info: |
    ClassElement :
      PrivateMethodDefinition

    MethodDefinition :
      AsyncGeneratorMethod

    Async Generator Function Definitions

    AsyncGeneratorMethod :
      async [no LineTerminator here] * # PropertyName ( UniqueFormalParameters ) { AsyncGeneratorBody }


    IdentifierReference : Identifier

    It is a Syntax Error if this production has a [Yield] parameter and
    StringValue of Identifier is "yield".

---*/
$DONOTEVALUATE();


class C { async *#gen() {
    void yield;
}}
