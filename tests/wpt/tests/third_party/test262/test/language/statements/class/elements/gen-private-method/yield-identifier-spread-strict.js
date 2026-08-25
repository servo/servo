// This file was procedurally generated from the following sources:
// - src/generators/yield-identifier-spread-strict.case
// - src/generators/default/class-decl-private-method.template
/*---
description: It's an early error if the AssignmentExpression is a function body with yield as an identifier in strict mode. (Generator private method as a ClassDeclaration element)
esid: prod-GeneratorPrivateMethod
features: [object-spread, generators, class-methods-private]
flags: [generated, onlyStrict]
negative:
  phase: parse
  type: SyntaxError
info: |
    ClassElement :
      PrivateMethodDefinition

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

class C {
    *#gen() {
        callCount += 1;
        return {
             ...(function() {
                var yield;
                throw new Test262Error();
             }()),
          }
    }
    get gen() { return this.#gen; }
}

const c = new C();

// Test the private fields do not appear as properties before set to value
assert(
  !Object.prototype.hasOwnProperty.call(C.prototype, "#gen"),
  "Private field '#gen' does not appear as an own property on C prototype"
);
assert(
  !Object.prototype.hasOwnProperty.call(C, "#gen"),
  "Private field '#gen' does not appear as an own property on C constructor"
);
assert(
  !Object.prototype.hasOwnProperty.call(c, "#gen"),
  "Private field '#gen' does not appear as an own property on C instance"
);

var iter = c.gen();



assert.sameValue(callCount, 1);

// Test the private fields do not appear as properties after set to value
assert(
  !Object.prototype.hasOwnProperty.call(C.prototype, "#gen"),
  "Private field '#gen' does not appear as an own property on C prototype"
);
assert(
  !Object.prototype.hasOwnProperty.call(C, "#gen"),
  "Private field '#gen' does not appear as an own property on C constructor"
);
assert(
  !Object.prototype.hasOwnProperty.call(c, "#gen"),
  "Private field '#gen' does not appear as an own property on C instance"
);
