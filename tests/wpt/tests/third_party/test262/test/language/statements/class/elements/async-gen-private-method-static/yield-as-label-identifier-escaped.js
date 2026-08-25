// This file was procedurally generated from the following sources:
// - src/async-generators/yield-as-label-identifier-escaped.case
// - src/async-generators/syntax/async-class-decl-static-private-method.template
/*---
description: yield is a reserved keyword within generator function bodies and may not be used as a label identifier. (Static async generator private method as a ClassDeclaration element)
esid: prod-AsyncGeneratorMethod
features: [async-iteration, class-static-methods-private]
flags: [generated]
negative:
  phase: parse
  type: SyntaxError
info: |
    ClassElement :
      static PrivateMethodDefinition

    MethodDefinition :
      AsyncGeneratorMethod

    Async Generator Function Definitions

    AsyncGeneratorMethod :
      async [no LineTerminator here] * # PropertyName ( UniqueFormalParameters ) { AsyncGeneratorBody }


    LabelIdentifier : Identifier

    It is a Syntax Error if this production has a [Yield] parameter and
    StringValue of Identifier is "yield".

---*/
$DONOTEVALUATE();


class C { static async *#gen() {
    yi\u0065ld: ;
}}
