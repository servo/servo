// This file was procedurally generated from the following sources:
// - src/generators/yield-identifier-strict.case
// - src/generators/default/class-expr-method.template
/*---
description: It's an early error if the generator body has another function body with yield as an identifier in strict mode. (Generator method as a ClassExpression element)
esid: prod-GeneratorMethod
features: [generators]
flags: [generated, onlyStrict]
negative:
  phase: parse
  type: SyntaxError
info: |
    ClassElement :
      MethodDefinition

    MethodDefinition :
      GeneratorMethod

    14.4 Generator Function Definitions

    GeneratorMethod :
      * PropertyName ( UniqueFormalParameters ) { GeneratorBody }

---*/
$DONOTEVALUATE();

var callCount = 0;

var C = class {*gen() {
    callCount += 1;
    (function() {
        var yield;
        throw new Test262Error();
      }())
}}

var gen = C.prototype.gen;

var iter = gen();



assert.sameValue(callCount, 1);
