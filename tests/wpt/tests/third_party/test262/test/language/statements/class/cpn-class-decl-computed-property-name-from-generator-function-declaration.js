// This file was procedurally generated from the following sources:
// - src/computed-property-names/computed-property-name-from-generator-function-declaration.case
// - src/computed-property-names/evaluation/class-declaration.template
/*---
description: Computed property name from generator function (ComputedPropertyName in ClassDeclaration)
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
function * g() { return 1; }


class C {
  [g()]() {
    return 1;
  }
  static [g()]() {
    return 1;
  }
};

let c = new C();

assert.sameValue(
  c[g()](),
  1
);
assert.sameValue(
  C[g()](),
  1
);
assert.sameValue(
  c[String(g())](),
  1
);
assert.sameValue(
  C[String(g())](),
  1
);
