// This file was procedurally generated from the following sources:
// - src/computed-property-names/computed-property-name-from-integer-e-notational-literal.case
// - src/computed-property-names/evaluation/class-expression-accessors.template
/*---
description: Computed property name from numeric literal (ComputedPropertyName in ClassExpression)
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


let C = class {
  get [1]() {
    return 2;
  }

  set [1](v) {
    return 2;
  }

  static get [1]() {
    return 2;
  }

  static set [1](v) {
    return 2;
  }
};

let c = new C();

assert.sameValue(
  c[1],
  2
);
assert.sameValue(
  c[1] = 2,
  2
);

assert.sameValue(
  C[1],
  2
);
assert.sameValue(
  C[1] = 2,
  2
);
assert.sameValue(
  c[String(1)],
  2
);
assert.sameValue(
  c[String(1)] = 2,
  2
);

assert.sameValue(
  C[String(1)],
  2
);
assert.sameValue(
  C[String(1)] = 2,
  2
);
