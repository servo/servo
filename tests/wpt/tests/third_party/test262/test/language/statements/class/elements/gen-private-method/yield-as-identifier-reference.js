// This file was procedurally generated from the following sources:
// - src/generators/yield-as-identifier-reference.case
// - src/generators/syntax/class-decl-private-method.template
/*---
description: yield is a reserved keyword within generator function bodies and may not be used as an identifier reference. (Generator method as a ClassDeclaration element)
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


    IdentifierReference : Identifier

    It is a Syntax Error if this production has a [Yield] parameter and
    StringValue of Identifier is "yield".

---*/
$DONOTEVALUATE();

class C { *#gen() {
    void yield;
}}
