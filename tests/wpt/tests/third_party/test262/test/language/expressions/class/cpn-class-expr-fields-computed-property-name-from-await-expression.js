// This file was procedurally generated from the following sources:
// - src/computed-property-names/computed-property-name-from-await-expression.case
// - src/computed-property-names/evaluation/class-expression-fields.template
/*---
description: Computed property name from condition expression (ComputedPropertyName in ClassExpression)
esid: prod-ComputedPropertyName
features: [computed-property-names, top-level-await, class-fields-public, class-static-fields-public]
flags: [generated, async, module]
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
try {


let C = class {
  [await 9] = 9;

  static [await 9] = 9;
};

let c = new C();

assert.sameValue(
  c[await 9],
  9
);
assert.sameValue(
  C[await 9],
  9
);
assert.sameValue(
  c[String(await 9)],
  9
);
assert.sameValue(
  C[String(await 9)],
  9
);

} catch (e) {
  $DONE(e);
}
$DONE();
