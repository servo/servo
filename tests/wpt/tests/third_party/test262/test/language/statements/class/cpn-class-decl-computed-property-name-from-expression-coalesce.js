// This file was procedurally generated from the following sources:
// - src/computed-property-names/computed-property-name-from-expression-coalesce.case
// - src/computed-property-names/evaluation/class-declaration.template
/*---
description: Computed property name from coalesce (ComputedPropertyName in ClassDeclaration)
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
let x;


class C {
  [x ?? 1]() {
    return 2;
  }
  static [x ?? 1]() {
    return 2;
  }
};

let c = new C();

assert.sameValue(
  c[x ?? 1](),
  2
);
assert.sameValue(
  C[x ?? 1](),
  2
);
assert.sameValue(
  c[String(x ?? 1)](),
  2
);
assert.sameValue(
  C[String(x ?? 1)](),
  2
);

assert.sameValue(x, undefined);
