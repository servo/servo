// This file was procedurally generated from the following sources:
// - src/generators/yield-as-label-identifier-escaped.case
// - src/generators/syntax/class-expr-private-method.template
/*---
description: yield is a reserved keyword within generator function bodies and may not be used as a label identifier. (Generator private method as a ClassExpression element)
esid: prod-GeneratorMethod
features: [generators, class-methods-private]
flags: [generated]
negative:
  phase: parse
  type: SyntaxError
info: |
    ClassElement :
      PrivateMethodDefinition

    MethodDefinition :
      GeneratorMethod

    14.4 Generator Function Definitions

    GeneratorMethod :
      * # PropertyName ( UniqueFormalParameters ) { GeneratorBody }


    LabelIdentifier : Identifier

    It is a Syntax Error if this production has a [Yield] parameter and
    StringValue of Identifier is "yield".

---*/
$DONOTEVALUATE();

var C = class {*#gen() {
    yi\u0065ld: ;
}};
