// This file was procedurally generated from the following sources:
// - src/computed-property-names/computed-property-name-from-identifier.case
// - src/computed-property-names/evaluation/class-declaration-accessors.template
/*---
description: Computed property name from string literal (ComputedPropertyName in ClassDeclaration)
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
let x = 1;



class C {
  get [x]() {
    return '2';
  }

  set [x](v) {
    return '2';
  }

  static get [x]() {
    return '2';
  }

  static set [x](v) {
    return '2';
  }
};

let c = new C();

assert.sameValue(
  c[x],
  '2'
);
assert.sameValue(
  c[x] = '2',
  '2'
);

assert.sameValue(
  C[x],
  '2'
);
assert.sameValue(
  C[x] = '2',
  '2'
);
assert.sameValue(
  c[String(x)],
  '2'
);
assert.sameValue(
  c[String(x)] = '2',
  '2'
);

assert.sameValue(
  C[String(x)],
  '2'
);
assert.sameValue(
  C[String(x)] = '2',
  '2'
);
