// This file was procedurally generated from the following sources:
// - src/generators/yield-as-identifier-reference-escaped.case
// - src/generators/syntax/class-expr-static-private-method.template
/*---
description: yield is a reserved keyword within generator function bodies and may not be used as an identifier reference. (Static generator private method as a ClassExpression element)
esid: prod-GeneratorMethod
features: [generators, class-static-methods-private]
flags: [generated]
negative:
  phase: parse
  type: SyntaxError
info: |
    ClassElement :
      static PrivateMethodDefinition

    MethodDefinition :
      GeneratorMethod

    14.4 Generator Function Definitions

    GeneratorMethod :
      * # PropertyName ( UniqueFormalParameters ) { GeneratorBody }


    IdentifierReference : Identifier

    It is a Syntax Error if this production has a [Yield] parameter and
    StringValue of Identifier is "yield".

---*/
$DONOTEVALUATE();

var C = class { static *#gen() {
    void yi\u0065ld;
}};
