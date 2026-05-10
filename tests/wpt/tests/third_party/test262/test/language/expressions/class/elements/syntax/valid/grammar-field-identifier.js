// This file was procedurally generated from the following sources:
// - src/class-elements/grammar-field-identifier.case
// - src/class-elements/syntax/valid/cls-expr-elements-valid-syntax.template
/*---
description: Valid FieldDefinition, ClassElementName, PropertyName Syntax (class expression)
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

    PropertyName :
      LiteralPropertyName
      ComputedPropertyName

    LiteralPropertyName :
      IdentifierName
      StringLiteral
      NumericLiteral

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
  $;
  _;
  \u{6F};
  \u2118;
  ZW_\u200C_NJ;
  ZW_\u200D_J;
};
