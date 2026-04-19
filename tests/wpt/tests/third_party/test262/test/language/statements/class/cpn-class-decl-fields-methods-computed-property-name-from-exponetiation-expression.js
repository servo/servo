// This file was procedurally generated from the following sources:
// - src/computed-property-names/computed-property-name-from-exponetiation-expression.case
// - src/computed-property-names/evaluation/class-declaration-fields-methods.template
/*---
description: Computed property name from exponentiation expression (ComputedPropertyName in ClassExpression)
esid: prod-ComputedPropertyName
features: [computed-property-names, exponentiation, class-fields-public, class-static-fields-public]
flags: [generated]
info: |
    ClassExpression:
      classBindingIdentifier opt ClassTail

    ClassTail:
      ClassHeritage opt { ClassBody opt }

    ClassBody:
      ClassElementList

    ClassElementList:
      ClassElement

    ClassElement:
      MethodDefinition

    MethodDefinition:
      PropertyName ...
      get PropertyName ...
      set PropertyName ...

    PropertyName:
      ComputedPropertyName

    ComputedPropertyName:
      [ AssignmentExpression ]
---*/


let C = class {
  [2 ** 2] = () => {
    return 4;
  };

  static [2 ** 2] = () => {
    return 4;
  };
};

let c = new C();

assert.sameValue(
  c[2 ** 2](),
  4
);
assert.sameValue(
  C[2 ** 2](),
  4
);
assert.sameValue(
  c[String(2 ** 2)](),
  4
);
assert.sameValue(
  C[String(2 ** 2)](),
  4
);
