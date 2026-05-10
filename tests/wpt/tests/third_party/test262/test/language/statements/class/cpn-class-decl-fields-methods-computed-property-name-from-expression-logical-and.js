// This file was procedurally generated from the following sources:
// - src/computed-property-names/computed-property-name-from-expression-logical-and.case
// - src/computed-property-names/evaluation/class-declaration-fields-methods.template
/*---
description: Computed property name from logical and (ComputedPropertyName in ClassExpression)
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
let x = 0;


let C = class {
  [x && 1] = () => {
    return 2;
  };

  static [x && 1] = () => {
    return 2;
  };
};

let c = new C();

assert.sameValue(
  c[x && 1](),
  2
);
assert.sameValue(
  C[x && 1](),
  2
);
assert.sameValue(
  c[String(x && 1)](),
  2
);
assert.sameValue(
  C[String(x && 1)](),
  2
);

assert.sameValue(x, 0);
