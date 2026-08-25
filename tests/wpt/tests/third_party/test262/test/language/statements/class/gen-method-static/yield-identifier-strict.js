// This file was procedurally generated from the following sources:
// - src/generators/yield-identifier-strict.case
// - src/generators/default/class-decl-static-method.template
/*---
description: It's an early error if the generator body has another function body with yield as an identifier in strict mode. (Static generator method as a ClassDeclaration element)
esid: prod-GeneratorMethod
features: [generators]
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

---*/
$DONOTEVALUATE();

var callCount = 0;

class C {static *gen() {
    callCount += 1;
    (function() {
        var yield;
        throw new Test262Error();
      }())
}}

var gen = C.gen;

var iter = gen();



assert.sameValue(callCount, 1);
