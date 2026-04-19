// This file was procedurally generated from the following sources:
// - src/generators/yield-identifier-strict.case
// - src/generators/default/expression-named.template
/*---
description: It's an early error if the generator body has another function body with yield as an identifier in strict mode. (Named generator expression)
esid: prod-GeneratorExpression
features: [generators]
flags: [generated, onlyStrict]
negative:
  phase: parse
  type: SyntaxError
info: |
    14.4 Generator Function Definitions

    GeneratorExpression:
      function * BindingIdentifier opt ( FormalParameters ) { GeneratorBody }

---*/
$DONOTEVALUATE();

var callCount = 0;

var gen = function *g() {
  callCount += 1;
  (function() {
      var yield;
      throw new Test262Error();
    }())
};

var iter = gen();



assert.sameValue(callCount, 1);
