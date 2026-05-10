// This file was procedurally generated from the following sources:
// - src/computed-property-names/computed-property-name-from-function-declaration.case
// - src/computed-property-names/evaluation/class-declaration.template
/*---
description: Computed property name from function (ComputedPropertyName in ClassDeclaration)
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
function f() {}


class C {
  [f()]() {
    return 1;
  }
  static [f()]() {
    return 1;
  }
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
