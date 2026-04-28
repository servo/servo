// This file was procedurally generated from the following sources:
// - src/class-elements/init-value-defined-after-class.case
// - src/class-elements/default/cls-decl.template
/*---
description: The initializer value is defined after the class evaluation (field definitions in a class declaration)
esid: prod-FieldDefinition
features: [class-fields-public, computed-property-names, class]
flags: [generated]
includes: [propertyHelper.js]
info: |
    [[Construct]] ( argumentsList, newTarget)

    8. If kind is "base", then
      a. Perform OrdinaryCallBindThis(F, calleeContext, thisArgument).
      b. Let result be InitializeInstanceFields(thisArgument, F).
      ...
    ...
    11. Let result be OrdinaryCallEvaluateBody(F, argumentsList).
    ...

---*/
var x = false;


class C {
  [x] = x;
}

var c1 = new C();

x = true;
var c2 = new C();

verifyProperty(c1, "false", {
  value: false,
  enumerable: true,
  configurable: true,
  writable: true,
});
verifyProperty(c2, "false", {
  value: true,
  enumerable: true,
  configurable: true,
  writable: true,
});

assert.sameValue(c1.hasOwnProperty("true"), false);
assert.sameValue(c2.hasOwnProperty("true"), false);
