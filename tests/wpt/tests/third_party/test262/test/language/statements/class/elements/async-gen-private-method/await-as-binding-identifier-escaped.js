// This file was procedurally generated from the following sources:
// - src/async-generators/await-as-binding-identifier-escaped.case
// - src/async-generators/syntax/async-class-decl-private-method.template
/*---
description: await is a reserved keyword within generator function bodies and may not be used as a binding identifier. (Async Generator private method as a ClassDeclaration element)
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


    BindingIdentifier : Identifier

    It is a Syntax Error if this production has a [Await] parameter and
    StringValue of Identifier is "await".

---*/
$DONOTEVALUATE();


class C { async *#gen() {
    var \u0061wait;
}}
