// This file was procedurally generated from the following sources:
// - src/computed-property-names/computed-property-name-from-function-expression.case
// - src/computed-property-names/evaluation/class-declaration-fields.template
/*---
description: Computed property name from function expression (ComputedPropertyName in ClassExpression)
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
  [function () {}] = 1;

  static [function () {}] = 1;
};

let c = new C();

assert.sameValue(
  c[function () {}],
  1
);
assert.sameValue(
  C[function () {}],
  1
);
assert.sameValue(
  c[String(function () {})],
  1
);
assert.sameValue(
  C[String(function () {})],
  1
);
