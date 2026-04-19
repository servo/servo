// This file was procedurally generated from the following sources:
// - src/accessor-names/private-escape-sequence-ZWNJ.case
// - src/accessor-names/private/cls-private-decl-inst.template
/*---
description: Private IdentifierName - ZWNJ (Class declaration, private instance method)
features: [class, class-methods-private]
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

var stringSet;

class C {
  get #ZW_\u200C_NJ() { return 'get string'; }
  set #ZW_\u200C_NJ(param) { stringSet = param; }

  getPrivateReference() {
    return this.#ZW_‌_NJ;
  }

  setPrivateReference(value) {
    this.#ZW_‌_NJ = value;
  }
};

var inst = new C();

assert.sameValue(inst.getPrivateReference(), 'get string');

inst.setPrivateReference('set string');
assert.sameValue(stringSet, 'set string');
