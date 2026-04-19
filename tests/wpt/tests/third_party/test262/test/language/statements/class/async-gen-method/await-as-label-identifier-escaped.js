// This file was procedurally generated from the following sources:
// - src/async-generators/await-as-label-identifier-escaped.case
// - src/async-generators/syntax/async-class-decl-method.template
/*---
description: await is a reserved keyword within generator function bodies and may not be used as a label identifier. (Async Generator method as a ClassDeclaration element)
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


    LabelIdentifier : Identifier

    It is a Syntax Error if this production has a [Await] parameter and
    StringValue of Identifier is "await".

---*/
$DONOTEVALUATE();


class C { async *gen() {
    \u0061wait: ;
}}
