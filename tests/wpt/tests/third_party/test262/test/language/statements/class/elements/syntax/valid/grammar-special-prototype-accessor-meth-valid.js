// This file was procedurally generated from the following sources:
// - src/class-elements/grammar-special-prototype-accessor-meth-valid.case
// - src/class-elements/syntax/valid/cls-decl-elements-valid-syntax.template
/*---
description: Accessor Methods can be named "prototype" (class declaration)
esid: prod-ClassElement
features: [class]
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

    MethodDefinition : get PropertyName ( ) { FunctionBody }

    [...]
    9. Let desc be the PropertyDescriptor { [[Get]]: closure, [[Enumerable]]: enumerable, [[Configurable]]: true }.
    10. Return ? DefinePropertyOrThrow(object, propKey, desc).

    MethodDefinition : set PropertyName ( PropertySetParameterList ) { FunctionBody }

    [...]
    8. Let desc be the PropertyDescriptor { [[Set]]: closure, [[Enumerable]]: enumerable, [[Configurable]]: true }.
    9. Return ? DefinePropertyOrThrow(object, propKey, desc).

---*/


class C {
  get prototype() { return 13; }
  set prototype(_) {}
}

assert(C.hasOwnProperty('prototype'));
assert(C.prototype.hasOwnProperty('prototype'));
assert.notSameValue(C.prototype.prototype, C.prototype);
assert.sameValue(C.prototype.prototype, 13);
verifyProperty(C.prototype, 'prototype', {
    enumerable: false,
    configurable: true,
});
