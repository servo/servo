// This file was procedurally generated from the following sources:
// - src/computed-property-names/computed-property-name-from-condition-expression-true.case
// - src/computed-property-names/evaluation/class-expression-fields.template
/*---
description: Computed property name from condition expression (ComputedPropertyName in ClassExpression)
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
  [true ? 1 : 2] = 2;

  static [true ? 1 : 2] = 2;
};

let c = new C();

assert.sameValue(
  c[true ? 1 : 2],
  2
);
assert.sameValue(
  C[true ? 1 : 2],
  2
);
assert.sameValue(
  c[String(true ? 1 : 2)],
  2
);
assert.sameValue(
  C[String(true ? 1 : 2)],
  2
);
