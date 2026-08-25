// This file was procedurally generated from the following sources:
// - src/class-elements/static-as-valid-static-field-assigned.case
// - src/class-elements/default/cls-decl.template
/*---
description: static is a valid name of a static field (field definitions in a class declaration)
esid: prod-FieldDefinition
features: [class-static-fields-public, class]
flags: [generated]
includes: [propertyHelper.js]
info: |
    ClassElement:
      ...
      static FieldDefinition ;

---*/


class C {
  static static = "test262";
}

verifyProperty(C, "static", {
  value: "test262",
  enumerable: true,
  writable: true,
  configurable: true
});

