// This file was procedurally generated from the following sources:
// - src/async-generators/yield-identifier-strict.case
// - src/async-generators/default/async-expression-named.template
/*---
description: It's an early error if the generator body has another function body with yield as an identifier in strict mode. (Named async generator expression)
esid: prod-AsyncGeneratorExpression
features: [async-iteration]
flags: [generated, onlyStrict]
negative:
  phase: parse
  type: SyntaxError
info: |
    Async Generator Function Definitions

    AsyncGeneratorExpression :
      async [no LineTerminator here] function * BindingIdentifier ( FormalParameters ) {
        AsyncGeneratorBody }

---*/
$DONOTEVALUATE();


var callCount = 0;

var gen = async function *g() {
  callCount += 1;
  (function() {
      var yield;
      throw new Test262Error();
    }())
};

var iter = gen();



assert.sameValue(callCount, 1);
