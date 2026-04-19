// This file was procedurally generated from the following sources:
// - src/class-elements/init-value-incremental.case
// - src/class-elements/default/cls-decl.template
/*---
description: The initializer value is defined during the class instatiation (field definitions in a class declaration)
esid: prod-FieldDefinition
features: [class-fields-public, computed-property-names, class]
flags: [generated]
includes: [propertyHelper.js]
info: |
    Runtime Semantics: ClassDefinitionEvaluation

    27. For each ClassElement e in order from elements
      ...
      d. Append to fieldRecords the elements of fields.
    ...
    33. Let result be InitializeStaticFields(F).
    ...

    [[Construct]] ( argumentsList, newTarget)

    8. If kind is "base", then
      a. Perform OrdinaryCallBindThis(F, calleeContext, thisArgument).
      b. Let result be InitializeInstanceFields(thisArgument, F).
      ...
    ...
    11. Let result be OrdinaryCallEvaluateBody(F, argumentsList).
    ...

---*/
var x = 1;


class C {
  [x++] = x++;
  [x++] = x++;
}

var c1 = new C();
var c2 = new C();

verifyProperty(c1, "1", {
  value: 3,
  enumerable: true,
  configurable: true,
  writable: true,
});

verifyProperty(c1, "2", {
  value: 4,
  enumerable: true,
  configurable: true,
  writable: true,
});

verifyProperty(c2, "1", {
  value: 5,
  enumerable: true,
  configurable: true,
  writable: true,
});

verifyProperty(c2, "2", {
  value: 6,
  enumerable: true,
  configurable: true,
  writable: true,
});
