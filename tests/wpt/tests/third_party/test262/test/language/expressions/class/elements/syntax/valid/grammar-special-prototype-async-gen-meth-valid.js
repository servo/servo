// This file was procedurally generated from the following sources:
// - src/class-elements/grammar-special-prototype-async-gen-meth-valid.case
// - src/class-elements/syntax/valid/cls-expr-elements-valid-syntax.template
/*---
description: Async Generator Methods can be named "prototype" (class expression)
esid: prod-ClassElement
features: [async-iteration, class]
flags: [generated]
includes: [propertyHelper.js]
info: |
    Runtime Semantics: ClassDefinitionEvaluation

    ClassTail : ClassHeritage_opt { ClassBody_opt }

    [...]
    6. Let proto be OrdinaryObjectCreate(protoParent).
    [...]
    14. Perform MakeConstructor(F, false, proto).
    [...]
    20. For each ClassElement m in order from methods, do
        a. If IsStatic of m is false, then
            i. Let status be PropertyDefinitionEvaluation of m with arguments proto and false.
    [...]

    Runtime Semantics: PropertyDefinitionEvaluation

    With parameters object and enumerable.

    AsyncGeneratorMethod : async * PropertyName ( UniqueFormalParameters ) { AsyncGeneratorBody }

    [...]
    10. Let desc be PropertyDescriptor { [[Value]]: closure, [[Writable]]: true, [[Enumerable]]: enumerable, [[Configurable]]: true }.
    11. Return ? DefinePropertyOrThrow(object, propKey, desc).

---*/


var C = class {
  async * prototype() {}
};

assert(C.hasOwnProperty('prototype'));
assert(C.prototype.hasOwnProperty('prototype'));
assert.notSameValue(C.prototype.prototype, C.prototype);
verifyProperty(C.prototype, 'prototype', {
    writable: true,
    enumerable: false,
    configurable: true,
});
