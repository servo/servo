// This file was procedurally generated from the following sources:
// - src/class-elements/grammar-fields-multi-line.case
// - src/class-elements/syntax/valid/cls-decl-elements-valid-syntax.template
/*---
description: Valid multi-line, multi-field (class declaration)
esid: prod-ClassElement
features: [class-fields-public, class]
flags: [generated]
info: |
    ClassElement :
      MethodDefinition
      static MethodDefinition
      FieldDefinition ;
      ;

    FieldDefinition :
      ClassElementName Initializer _opt

    ClassElementName :
      PropertyName
      PrivateName

---*/


class C {
  x
  y
}
