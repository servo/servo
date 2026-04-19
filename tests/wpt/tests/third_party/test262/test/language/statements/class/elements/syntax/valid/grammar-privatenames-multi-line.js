// This file was procedurally generated from the following sources:
// - src/class-elements/grammar-privatenames-multi-line.case
// - src/class-elements/syntax/valid/cls-decl-elements-valid-syntax.template
/*---
description: SyntaxError (class declaration)
esid: prod-ClassElement
features: [class-fields-private, class]
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

    PrivateName ::
      # IdentifierName

---*/


class C {
  #x
  #y
}
