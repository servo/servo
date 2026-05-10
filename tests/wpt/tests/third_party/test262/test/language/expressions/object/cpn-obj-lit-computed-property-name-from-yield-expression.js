// This file was procedurally generated from the following sources:
// - src/computed-property-names/computed-property-name-from-yield-expression.case
// - src/computed-property-names/evaluation/object-literal.template
/*---
description: Computed property name from yield expression (ComputedPropertyName in ObjectLiteral)
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
function * g() {


let o = {
  [yield 9]: 9
};

assert.sameValue(
  o[yield 9],
  9
);
assert.sameValue(
  o[String(yield 9)],
  9
);

}
var iter = g();
while (iter.next().done === false) ;
