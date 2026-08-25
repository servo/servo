// This file was procedurally generated from the following sources:
// - src/class-elements/grammar-field-identifier-invalid-ues-error.case
// - src/class-elements/syntax/invalid/cls-decl-elements-invalid-syntax.template
/*---
description: Invalid FieldDefinition, ClassElementName, PropertyName Syntax (class declaration)
esid: prod-ClassElement
features: [class-fields-public, class]
flags: [generated]
negative:
  phase: parse
  type: SyntaxError
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


$DONOTEVALUATE();

class C {
  \u0000;
}
