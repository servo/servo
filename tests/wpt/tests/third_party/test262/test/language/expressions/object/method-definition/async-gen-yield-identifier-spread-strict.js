// This file was procedurally generated from the following sources:
// - src/async-generators/yield-identifier-spread-strict.case
// - src/async-generators/default/async-obj-method.template
/*---
description: It's an early error if the AssignmentExpression is a function body with yield as an identifier in strict mode. (Async generator method)
esid: prod-AsyncGeneratorMethod
features: [object-spread, async-iteration]
flags: [generated, onlyStrict]
negative:
  phase: parse
  type: SyntaxError
info: |
    Async Generator Function Definitions

    AsyncGeneratorMethod :
      async [no LineTerminator here] * PropertyName ( UniqueFormalParameters ) { AsyncGeneratorBody }


    Spread Properties

    PropertyDefinition[Yield]:
      (...)
      ...AssignmentExpression[In, ?Yield]

---*/
$DONOTEVALUATE();

var callCount = 0;

var gen = {
  async *method() {
    callCount += 1;
    return {
         ...(function() {
            var yield;
            throw new Test262Error();
         }()),
      }
  }
}.method;

var iter = gen();



assert.sameValue(callCount, 1);
