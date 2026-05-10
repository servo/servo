// This file was procedurally generated from the following sources:
// - src/async-generators/yield-identifier-spread-strict.case
// - src/async-generators/default/async-class-decl-method.template
/*---
description: It's an early error if the AssignmentExpression is a function body with yield as an identifier in strict mode. (Async Generator method as a ClassDeclaration element)
esid: prod-AsyncGeneratorMethod
features: [object-spread, async-iteration]
flags: [generated, onlyStrict]
negative:
  phase: parse
  type: SyntaxError
info: |
    ClassElement :
      MethodDefinition

    MethodDefinition :
      AsyncGeneratorMethod

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

class C { async *gen() {
    callCount += 1;
    return {
         ...(function() {
            var yield;
            throw new Test262Error();
         }()),
      }
}}

var gen = C.prototype.gen;

var iter = gen();



assert.sameValue(callCount, 1);
