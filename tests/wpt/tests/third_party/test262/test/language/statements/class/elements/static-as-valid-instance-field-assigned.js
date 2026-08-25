// This file was procedurally generated from the following sources:
// - src/class-elements/static-as-valid-instance-field-assigned.case
// - src/class-elements/default/cls-decl.template
/*---
description: static is a valid name of an instance field (field definitions in a class declaration)
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


class C {
  static = "foo";
}

let c = new C();

verifyProperty(c, "static", {
  value: "foo",
  enumerable: true,
  writable: true,
  configurable: true
});

