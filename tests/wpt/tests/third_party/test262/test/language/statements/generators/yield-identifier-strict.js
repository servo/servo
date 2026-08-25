// This file was procedurally generated from the following sources:
// - src/generators/yield-identifier-strict.case
// - src/generators/default/declaration.template
/*---
description: It's an early error if the generator body has another function body with yield as an identifier in strict mode. (Generator Function declaration)
esid: prod-GeneratorDeclaration
features: [generators]
flags: [generated, onlyStrict]
negative:
  phase: parse
  type: SyntaxError
info: |
    14.4 Generator Function Definitions

    GeneratorDeclaration :
      function * BindingIdentifier ( FormalParameters ) { GeneratorBody }

---*/
$DONOTEVALUATE();

var callCount = 0;

function *gen() {
  callCount += 1;
  (function() {
      var yield;
      throw new Test262Error();
    }())
}

var iter = gen();



assert.sameValue(callCount, 1);
