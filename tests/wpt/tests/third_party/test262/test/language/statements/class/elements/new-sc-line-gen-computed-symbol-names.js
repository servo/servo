// This file was procedurally generated from the following sources:
// - src/class-elements/computed-symbol-names.case
// - src/class-elements/productions/cls-decl-new-sc-line-generator.template
/*---
description: Computed property symbol names (field definitions followed by a method in a new line with a semicolon)
esid: prod-FieldDefinition
features: [class-fields-public, Symbol, computed-property-names, class, generators]
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
var x = Symbol();
var y = Symbol();



class C {
  [x]; [y] = 42;
  *m() { return 42; }
  
}

var c = new C();

assert.sameValue(c.m().next().value, 42);
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
  !Object.prototype.hasOwnProperty.call(C.prototype, x),
  "Symbol x doesn't appear as an own property on C prototype"
);
assert(
  !Object.prototype.hasOwnProperty.call(C, x),
  "Symbol x doesn't appear as an own property on C constructor"
);

verifyProperty(c, x, {
  value: undefined,
  enumerable: true,
  writable: true,
  configurable: true
});

assert(
  !Object.prototype.hasOwnProperty.call(C.prototype, y),
  "Symbol y doesn't appear as an own property on C prototype"
);
assert(
  !Object.prototype.hasOwnProperty.call(C, y),
  "Symbol y doesn't appear as an own property on C constructor"
);

verifyProperty(c, y, {
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
  !Object.prototype.hasOwnProperty.call(C.prototype, "y"),
  "y doesn't appear as an own property on C prototype"
);
assert(
  !Object.prototype.hasOwnProperty.call(C, "y"),
  "y doesn't appear as an own property on C constructor"
);
assert(
  !Object.prototype.hasOwnProperty.call(c, "y"),
  "y doesn't appear as an own property on C instance"
);
