// This file was procedurally generated from the following sources:
// - src/class-elements/string-literal-names.case
// - src/class-elements/productions/cls-expr-wrapped-in-sc.template
/*---
description: String literal names (fields definition wrapped in semicolons)
esid: prod-FieldDefinition
features: [class-fields-public, class]
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


var C = class {
  ;;;;
  ;;;;;;'a'; "b"; 'c' = 39;
  "d" = 42;;;;;;;
  ;;;;
  
}

var c = new C();

assert(
  !Object.prototype.hasOwnProperty.call(C.prototype, "a"),
  "a does not appear as an own property on C prototype"
);
assert(
  !Object.prototype.hasOwnProperty.call(C, "a"),
  "a does not appear as an own property on C constructor"
);

verifyProperty(c, "a", {
  value: undefined,
  enumerable: true,
  writable: true,
  configurable: true
});

assert(
  !Object.prototype.hasOwnProperty.call(C.prototype, "b"),
  "b does not appear as an own property on C prototype"
);
assert(
  !Object.prototype.hasOwnProperty.call(C, "b"),
  "b does not appear as an own property on C constructor"
);

verifyProperty(c, "b", {
  value: undefined,
  enumerable: true,
  writable: true,
  configurable: true
});

assert(
  !Object.prototype.hasOwnProperty.call(C.prototype, "c"),
  "c does not appear as an own property on C prototype"
);
assert(
  !Object.prototype.hasOwnProperty.call(C, "c"),
  "c does not appear as an own property on C constructor"
);

verifyProperty(c, "c", {
  value: 39,
  enumerable: true,
  writable: true,
  configurable: true
});

assert(
  !Object.prototype.hasOwnProperty.call(C.prototype, "d"),
  "d does not appear as an own property on C prototype"
);
assert(
  !Object.prototype.hasOwnProperty.call(C, "d"),
  "d does not appear as an own property on C constructor"
);

verifyProperty(c, "d", {
  value: 42,
  enumerable: true,
  writable: true,
  configurable: true
});
