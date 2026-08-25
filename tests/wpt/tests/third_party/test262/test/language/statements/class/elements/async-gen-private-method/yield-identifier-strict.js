// This file was procedurally generated from the following sources:
// - src/async-generators/yield-identifier-strict.case
// - src/async-generators/default/async-class-decl-private-method.template
/*---
description: It's an early error if the generator body has another function body with yield as an identifier in strict mode. (Async Generator method as a ClassDeclaration element)
esid: prod-AsyncGeneratorPrivateMethod
features: [async-iteration, class-methods-private]
flags: [generated, onlyStrict]
negative:
  phase: parse
  type: SyntaxError
info: |
    ClassElement :
      PrivateMethodDefinition

    MethodDefinition :
      AsyncGeneratorMethod

    Async Generator Function Definitions

    AsyncGeneratorMethod :
      async [no LineTerminator here] * PropertyName ( UniqueFormalParameters ) { AsyncGeneratorBody }

---*/
$DONOTEVALUATE();


var callCount = 0;

class C {
    async *#gen() {
        callCount += 1;
        (function() {
            var yield;
            throw new Test262Error();
          }())
    }
    get gen() { return this.#gen; }
}

const c = new C();

// Test the private fields do not appear as properties before set to value
assert(
  !Object.prototype.hasOwnProperty.call(C.prototype, "#gen"),
  "#gen does not appear as an own property on C prototype"
);
assert(
  !Object.prototype.hasOwnProperty.call(C, "#gen"),
  "#gen does not appear as an own property on C constructor"
);
assert(
  !Object.prototype.hasOwnProperty.call(c, "#gen"),
  "#gen does not appear as an own property on C instance"
);

var iter = c.gen();



assert.sameValue(callCount, 1);

// Test the private fields do not appear as properties after set to value
assert(
  !Object.prototype.hasOwnProperty.call(C.prototype, "#gen"),
  "#gen does not appear as an own property on C prototype"
);
assert(
  !Object.prototype.hasOwnProperty.call(C, "#gen"),
  "#gen does not appear as an own property on C constructor"
);
assert(
  !Object.prototype.hasOwnProperty.call(c, "#gen"),
  "#gen does not appear as an own property on C instance"
);
