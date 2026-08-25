// This file was procedurally generated from the following sources:
// - src/computed-property-names/computed-property-name-from-identifier.case
// - src/computed-property-names/evaluation/object-literal.template
/*---
description: Computed property name from string literal (ComputedPropertyName in ObjectLiteral)
esid: prod-ComputedPropertyName
features: [computed-property-names]
flags: [generated]
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
let x = 1;



let o = {
  [x]: '2'
};

assert.sameValue(
  o[x],
  '2'
);
assert.sameValue(
  o[String(x)],
  '2'
);
