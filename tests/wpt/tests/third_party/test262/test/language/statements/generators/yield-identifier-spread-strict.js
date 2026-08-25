// This file was procedurally generated from the following sources:
// - src/generators/yield-identifier-spread-strict.case
// - src/generators/default/declaration.template
/*---
description: It's an early error if the AssignmentExpression is a function body with yield as an identifier in strict mode. (Generator Function declaration)
esid: prod-GeneratorDeclaration
features: [object-spread, generators]
flags: [generated, onlyStrict]
negative:
  phase: parse
  type: SyntaxError
info: |
    14.4 Generator Function Definitions

    GeneratorDeclaration :
      function * BindingIdentifier ( FormalParameters ) { GeneratorBody }


    Spread Properties

    PropertyDefinition[Yield]:
      (...)
      ...AssignmentExpression[In, ?Yield]

---*/
$DONOTEVALUATE();

var callCount = 0;

function *gen() {
  callCount += 1;
  return {
       ...(function() {
          var yield;
          throw new Test262Error();
       }()),
    }
}

var iter = gen();



assert.sameValue(callCount, 1);
