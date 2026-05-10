// This file was procedurally generated from the following sources:
// - src/computed-property-names/computed-property-name-from-function-declaration.case
// - src/computed-property-names/evaluation/class-declaration-fields-methods.template
/*---
description: Computed property name from function (ComputedPropertyName in ClassExpression)
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
function f() {}


let C = class {
  [f()] = () => {
    return 1;
  };

  static [f()] = () => {
    return 1;
  };
};

let c = new C();

assert.sameValue(
  c[f()](),
  1
);
assert.sameValue(
  C[f()](),
  1
);
assert.sameValue(
  c[String(f())](),
  1
);
assert.sameValue(
  C[String(f())](),
  1
);
