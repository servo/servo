// This file was procedurally generated from the following sources:
// - src/generators/yield-identifier-spread-strict.case
// - src/generators/default/class-expr-static-method.template
/*---
description: It's an early error if the AssignmentExpression is a function body with yield as an identifier in strict mode. (Static generator method as a ClassExpression element)
esid: prod-GeneratorMethod
features: [object-spread, generators]
flags: [generated, onlyStrict]
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


    Spread Properties

    PropertyDefinition[Yield]:
      (...)
      ...AssignmentExpression[In, ?Yield]

---*/
$DONOTEVALUATE();

var callCount = 0;

var C = class { static *gen() {
    callCount += 1;
    return {
         ...(function() {
            var yield;
            throw new Test262Error();
         }()),
      }
}}

var gen = C.gen;

var iter = gen();



assert.sameValue(callCount, 1);
