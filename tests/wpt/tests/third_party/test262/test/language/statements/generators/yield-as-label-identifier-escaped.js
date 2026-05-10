// This file was procedurally generated from the following sources:
// - src/generators/yield-as-label-identifier-escaped.case
// - src/generators/syntax/declaration.template
/*---
description: yield is a reserved keyword within generator function bodies and may not be used as a label identifier. (Generator Function declaration)
esid: prod-GeneratorDeclaration
features: [generators]
flags: [generated]
negative:
  phase: parse
  type: SyntaxError
info: |
    14.4 Generator Function Definitions

    GeneratorDeclaration :
      function * BindingIdentifier ( FormalParameters ) { GeneratorBody }


    LabelIdentifier : Identifier

    It is a Syntax Error if this production has a [Yield] parameter and
    StringValue of Identifier is "yield".

---*/
$DONOTEVALUATE();

function *gen() {
  yi\u0065ld: ;
}
