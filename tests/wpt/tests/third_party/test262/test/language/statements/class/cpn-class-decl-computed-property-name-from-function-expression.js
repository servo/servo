// This file was procedurally generated from the following sources:
// - src/computed-property-names/computed-property-name-from-function-expression.case
// - src/computed-property-names/evaluation/class-declaration.template
/*---
description: Computed property name from function expression (ComputedPropertyName in ClassDeclaration)
esid: prod-ComputedPropertyName
features: [computed-property-names]
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


class C {
  [function () {}]() {
    return 1;
  }
  static [function () {}]() {
    return 1;
  }
};

let c = new C();

assert.sameValue(
  c[function () {}](),
  1
);
assert.sameValue(
  C[function () {}](),
  1
);
assert.sameValue(
  c[String(function () {})](),
  1
);
assert.sameValue(
  C[String(function () {})](),
  1
);
