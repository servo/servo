// This file was procedurally generated from the following sources:
// - src/class-elements/grammar-privatename-classelementname-initializer-alt.case
// - src/class-elements/syntax/valid/cls-decl-elements-valid-syntax.template
/*---
description: Valid PrivateName = Initializer Syntax (class declaration)
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

    IdentifierPart::
      UnicodeIDContinue
      $
      \ UnicodeEscapeSequence
      <ZWNJ> <ZWJ>

    UnicodeIDStart::
      any Unicode code point with the Unicode property "ID_Start"

    UnicodeIDContinue::
      any Unicode code point with the Unicode property "ID_Continue"


    NOTE 3
    The sets of code points with Unicode properties "ID_Start" and
    "ID_Continue" include, respectively, the code points with Unicode
    properties "Other_ID_Start" and "Other_ID_Continue".

---*/


class C {
  #$ = 1;
  #_ = 2;
  #\u{6F} = 3;
  #℘ = 4;      // DO NOT CHANGE THE NAME OF THIS FIELD
  #ZW_‌_NJ = 5; // DO NOT CHANGE THE NAME OF THIS FIELD
  #ZW_‍_J = 6;  // DO NOT CHANGE THE NAME OF THIS FIELD
}
