// This file was procedurally generated from the following sources:
// - src/computed-property-names/computed-property-name-from-async-arrow-function-expression.case
// - src/computed-property-names/evaluation/class-declaration-fields-methods.template
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
  [async () => {}] = () => {
    return 1;
  };

  static [async () => {}] = () => {
    return 1;
  };
};

let c = new C();

assert.sameValue(
  c[async () => {}](),
  1
);
assert.sameValue(
  C[async () => {}](),
  1
);
assert.sameValue(
  c[String(async () => {})](),
  1
);
assert.sameValue(
  C[String(async () => {})](),
  1
);
