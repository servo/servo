// This file was procedurally generated from the following sources:
// - src/generators/yield-as-binding-identifier-escaped.case
// - src/generators/syntax/class-expr-static-method.template
/*---
description: yield is a reserved keyword within generator function bodies and may not be used as a binding identifier. (Static generator method as a ClassExpression element)
esid: prod-GeneratorMethod
features: [generators]
flags: [generated]
negative:
  phase: parse
  type: SyntaxError
info: |
    ClassElement :
      static MethodDefinition

    MethodDefinition :
      GeneratorMethod

    14.4 Generator Function Definitions

    GeneratorMethod :
      * PropertyName ( UniqueFormalParameters ) { GeneratorBody }


    BindingIdentifier : Identifier

    It is a Syntax Error if this production has a [Yield] parameter and
    StringValue of Identifier is "yield".

---*/
$DONOTEVALUATE();

var C = class { static *gen() {
    var yi\u0065ld;
}};
