// This file was procedurally generated from the following sources:
// - src/class-elements/redeclaration.case
// - src/class-elements/default/cls-decl.template
/*---
description: Redeclaration of public fields with the same name (field definitions in a class declaration)
esid: prod-FieldDefinition
features: [class-fields-public, computed-property-names, class]
flags: [generated]
includes: [propertyHelper.js, compareArray.js]
info: |
    2.13.2 Runtime Semantics: ClassDefinitionEvaluation

    ...
    30. Set the value of F's [[Fields]] internal slot to fieldRecords.
    ...

    2.14 [[Construct]] ( argumentsList, newTarget)

    ...
    8. If kind is "base", then
      ...
      b. Let result be InitializeInstanceFields(thisArgument, F).
    ...

    2.9 InitializeInstanceFields ( O, constructor )

    3. Let fieldRecords be the value of constructor's [[Fields]] internal slot.
    4. For each item fieldRecord in order from fieldRecords,
      a. If fieldRecord.[[static]] is false, then
        i. Perform ? DefineField(O, fieldRecord).

---*/
var x = [];


class C {
  y = (x.push("a"), "old_value");
  ["y"] = (x.push("b"), "another_value");
  "y" = (x.push("c"), "same_value");
  y = (x.push("d"), "same_value");
}

var c = new C();

assert(
  !Object.prototype.hasOwnProperty.call(C.prototype, "y"),
  "y does not appear as an own property on C prototype"
);
assert(
  !Object.prototype.hasOwnProperty.call(C, "y"),
  "y does not appear as an own property on C constructor"
);

verifyProperty(c, "y", {
  value: "same_value",
  enumerable: true,
  writable: true,
  configurable: true
});

assert.compareArray(x, ["a", "b", "c", "d"]);
