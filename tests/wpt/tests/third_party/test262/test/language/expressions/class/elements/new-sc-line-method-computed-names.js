// This file was procedurally generated from the following sources:
// - src/class-elements/computed-names.case
// - src/class-elements/productions/cls-expr-new-sc-line-method.template
/*---
description: Computed property names (field definitions followed by a method in a new line with a semicolon)
esid: prod-FieldDefinition
features: [class-fields-public, computed-property-names, class]
flags: [generated]
includes: [propertyHelper.js]
info: |
    ClassElement:
      ...
      FieldDefinition ;

    FieldDefinition:
      ClassElementName Initializer_opt

    ClassElementName:
      PropertyName

---*/
var x = "b";



var C = class {
  [x] = 42; [10] = "meep"; ["not initialized"];
  m() { return 42; }
  
}

var c = new C();

assert.sameValue(c.m(), 42);
assert.sameValue(c.m, C.prototype.m);
assert(
  !Object.prototype.hasOwnProperty.call(c, "m"),
  "m doesn't appear as an own property on the C instance"
);

verifyProperty(C.prototype, "m", {
  enumerable: false,
  configurable: true,
  writable: true,
});

assert(
  !Object.prototype.hasOwnProperty.call(C.prototype, "b"),
  "b doesn't appear as an own property on C prototype"
);
assert(
  !Object.prototype.hasOwnProperty.call(C, "b"),
  "b doesn't appear as an own property on C constructor"
);

verifyProperty(c, "b", {
  value: 42,
  enumerable: true,
  writable: true,
  configurable: true
});

assert(
  !Object.prototype.hasOwnProperty.call(C.prototype, "x"),
  "x doesn't appear as an own property on C prototype"
);
assert(
  !Object.prototype.hasOwnProperty.call(C, "x"),
  "x doesn't appear as an own property on C constructor"
);
assert(
  !Object.prototype.hasOwnProperty.call(c, "x"),
  "x doesn't appear as an own property on C instance"
);

assert(
  !Object.prototype.hasOwnProperty.call(C.prototype, "10"),
  "10 doesn't appear as an own property on C prototype"
);
assert(
  !Object.prototype.hasOwnProperty.call(C, "10"),
  "10 doesn't appear as an own property on C constructor"
);

verifyProperty(c, "10", {
  value: "meep",
  enumerable: true,
  writable: true,
  configurable: true
});

assert(
  !Object.prototype.hasOwnProperty.call(C.prototype, "not initialized"),
  "'not initialized' doesn't appear as an own property on C prototype"
);
assert(
  !Object.prototype.hasOwnProperty.call(C, "not initialized"),
  "'not initialized' doesn't appear as an own property on C constructor"
);

verifyProperty(c, "not initialized", {
  value: undefined,
  enumerable: true,
  writable: true,
  configurable: true
});
