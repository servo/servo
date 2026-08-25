// This file was procedurally generated from the following sources:
// - src/computed-property-names/computed-property-name-from-decimal-literal.case
// - src/computed-property-names/evaluation/class-declaration-fields.template
/*---
description: Computed property name from decimal literal (ComputedPropertyName in ClassExpression)
esid: prod-ComputedPropertyName
features: [computed-property-names, class-fields-public, class-static-fields-public]
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
  [1.1] = 2;

  static [1.1] = 2;
};

let c = new C();

assert.sameValue(
  c[1.1],
  2
);
assert.sameValue(
  C[1.1],
  2
);
assert.sameValue(
  c[String(1.1)],
  2
);
assert.sameValue(
  C[String(1.1)],
  2
);
