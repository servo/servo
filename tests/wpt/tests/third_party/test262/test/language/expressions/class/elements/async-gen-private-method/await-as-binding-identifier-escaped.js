// This file was procedurally generated from the following sources:
// - src/async-generators/await-as-binding-identifier-escaped.case
// - src/async-generators/syntax/async-class-expr-private-method.template
/*---
description: await is a reserved keyword within generator function bodies and may not be used as a binding identifier. (Async generator private method as a ClassExpression element)
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


var C = class { async *#gen() {
    var \u0061wait;
}};
