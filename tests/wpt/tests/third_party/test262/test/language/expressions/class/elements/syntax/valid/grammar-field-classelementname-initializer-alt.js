// This file was procedurally generated from the following sources:
// - src/class-elements/grammar-field-classelementname-initializer-alt.case
// - src/class-elements/syntax/valid/cls-expr-elements-valid-syntax.template
/*---
description: FieldDefinition, ClassElementName, PropertyName = Initializer Syntax (class expression)
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


var C = class {
  $ = 1;
  _ = 2;
  \u{6F} = 3;
  ℘ = 4;      // DO NOT CHANGE THE NAME OF THIS FIELD
  ZW_‌_NJ = 5; // DO NOT CHANGE THE NAME OF THIS FIELD
  ZW_‍_J = 6;  // DO NOT CHANGE THE NAME OF THIS FIELD
};
