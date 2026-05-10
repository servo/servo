// This file was procedurally generated from the following sources:
// - src/class-elements/grammar-field-classelementname-initializer.case
// - src/class-elements/syntax/valid/cls-decl-elements-valid-syntax.template
/*---
description: FieldDefinition, ClassElementName, PropertyName = Initializer Syntax (class declaration)
esid: prod-ClassElement
features: [class-fields-public, class]
flags: [generated]
info: |
    ClassElement :
      ...
      FieldDefinition ;
      ;

    FieldDefinition :
      ClassElementName Initializer _opt

    ClassElementName :
      PropertyName
      PrivateName

    PropertyName :
      LiteralPropertyName
      ComputedPropertyName

    LiteralPropertyName :
      IdentifierName

    Initializer :
      = AssignmentExpression

    IdentifierName ::
      IdentifierStart
      IdentifierName IdentifierPart

    IdentifierStart ::
      UnicodeIDStart
      $
      _
      \ UnicodeEscapeSequence

    IdentifierPart ::
      UnicodeIDContinue
      $
      \ UnicodeEscapeSequence
      <ZWNJ> <ZWJ>

    UnicodeIDStart ::
      any Unicode code point with the Unicode property "ID_Start"

    UnicodeIDContinue ::
      any Unicode code point with the Unicode property "ID_Continue"


    NOTE 3
    The sets of code points with Unicode properties "ID_Start" and
    "ID_Continue" include, respectively, the code points with Unicode
    properties "Other_ID_Start" and "Other_ID_Continue".

---*/


class C {
  $ = 1;
  _ = 2;
  \u{6F} = 3;
  \u2118 = 4;
  ZW_\u200C_NJ = 5;
  ZW_\u200D_J = 6;
}
