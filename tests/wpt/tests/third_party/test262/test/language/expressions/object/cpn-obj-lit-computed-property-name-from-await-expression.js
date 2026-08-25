// This file was procedurally generated from the following sources:
// - src/computed-property-names/computed-property-name-from-await-expression.case
// - src/computed-property-names/evaluation/object-literal.template
/*---
description: Computed property name from condition expression (ComputedPropertyName in ObjectLiteral)
esid: prod-ComputedPropertyName
features: [computed-property-names, top-level-await]
flags: [generated, async, module]
info: |
    ObjectLiteral:
      { PropertyDefinitionList }

    PropertyDefinitionList:
      PropertyDefinition

    PropertyDefinition:
      PropertyName: AssignmentExpression

    PropertyName:
      ComputedPropertyName

    ComputedPropertyName:
      [ AssignmentExpression ]
---*/
try {


let o = {
  [await 9]: 9
};

assert.sameValue(
  o[await 9],
  9
);
assert.sameValue(
  o[String(await 9)],
  9
);

} catch (e) {
  $DONE(e);
}
$DONE();
