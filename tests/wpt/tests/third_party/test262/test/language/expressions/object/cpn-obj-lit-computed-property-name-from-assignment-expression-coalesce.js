// This file was procedurally generated from the following sources:
// - src/computed-property-names/computed-property-name-from-assignment-expression-coalesce.case
// - src/computed-property-names/evaluation/object-literal.template
/*---
description: Computed property name from assignment expression coalesce (ComputedPropertyName in ObjectLiteral)
esid: prod-ComputedPropertyName
features: [computed-property-names, logical-assignment-operators]
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
let x = null;


let o = {
  [x ??= 1]: 2
};

assert.sameValue(
  o[x ??= 1],
  2
);
assert.sameValue(
  o[String(x ??= 1)],
  2
);

assert.sameValue(x, 1);
