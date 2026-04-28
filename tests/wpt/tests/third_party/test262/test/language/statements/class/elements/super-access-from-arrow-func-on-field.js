// This file was procedurally generated from the following sources:
// - src/class-elements/super-access-from-arrow-func-on-field.case
// - src/class-elements/default/cls-decl.template
/*---
description: super inside arrow functions on field initializer resolves to class' super (field definitions in a class declaration)
esid: prod-FieldDefinition
features: [class-fields-public, class-static-fields-public, class]
flags: [generated]
info: |
    ClassElementName :
      PropertyName
      PrivateName

    SuperProperty:
      super[Expression]
      super.IdentifierName

---*/


class C {
  func = () => {
      super.prop = 'test262';
  }

  static staticFunc = () => {
      super.staticProp = 'static test262';
  }
}

let c = new C();
c.func();
assert.sameValue(c.prop, 'test262');

C.staticFunc();
assert.sameValue(C.staticProp, 'static test262');

