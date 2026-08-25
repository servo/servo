// This file was procedurally generated from the following sources:
// - src/computed-property-names/computed-property-name-from-condition-expression-false.case
// - src/computed-property-names/evaluation/class-declaration-accessors.template
/*---
description: Computed property name from condition expression (ComputedPropertyName in ClassDeclaration)
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
  get [false ? 1 : 2]() {
    return 1;
  }

  set [false ? 1 : 2](v) {
    return 1;
  }

  static get [false ? 1 : 2]() {
    return 1;
  }

  static set [false ? 1 : 2](v) {
    return 1;
  }
};

let c = new C();

assert.sameValue(
  c[false ? 1 : 2],
  1
);
assert.sameValue(
  c[false ? 1 : 2] = 1,
  1
);

assert.sameValue(
  C[false ? 1 : 2],
  1
);
assert.sameValue(
  C[false ? 1 : 2] = 1,
  1
);
assert.sameValue(
  c[String(false ? 1 : 2)],
  1
);
assert.sameValue(
  c[String(false ? 1 : 2)] = 1,
  1
);

assert.sameValue(
  C[String(false ? 1 : 2)],
  1
);
assert.sameValue(
  C[String(false ? 1 : 2)] = 1,
  1
);
