// This file was procedurally generated from the following sources:
// - src/class-elements/grammar-privatename-identifier-semantics-stringvalue.case
// - src/class-elements/productions/cls-expr-multiple-stacked-definitions.template
/*---
description: PrivateName Static Semantics, StringValue (multiple stacked fields definitions through ASI)
esid: prod-FieldDefinition
features: [class-fields-private, class, class-fields-public]
flags: [generated]
includes: [propertyHelper.js]
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


    1. Return the String value consisting of the sequence of code
      units corresponding to PrivateName. In determining the sequence
      any occurrences of \ UnicodeEscapeSequence are first replaced
      with the code point represented by the UnicodeEscapeSequence
      and then the code points of the entire PrivateName are converted
      to code units by UTF16Encoding (10.1.1) each code point.

---*/


var C = class {
  #\u{6F};
  #\u2118;
  #ZW_\u200C_NJ;
  #ZW_\u200D_J;
  foo = "foobar"
  bar = "barbaz";
  o(value) {
    this.#o = value;
    return this.#o;
  }
  ℘(value) {
    this.#℘ = value;
    return this.#℘;
  }
  ZW_‌_NJ(value) { // DO NOT CHANGE THE NAME OF THIS METHOD
    this.#ZW_‌_NJ = value;
    return this.#ZW_‌_NJ;
  }
  ZW_‍_J(value) { // DO NOT CHANGE THE NAME OF THIS METHOD
    this.#ZW_‍_J = value;
    return this.#ZW_‍_J;
  }
}

var c = new C();

assert.sameValue(c.foo, "foobar");
assert(
  !Object.prototype.hasOwnProperty.call(C, "foo"),
  "foo doesn't appear as an own property on the C constructor"
);
assert(
  !Object.prototype.hasOwnProperty.call(C.prototype, "foo"),
  "foo doesn't appear as an own property on the C prototype"
);

verifyProperty(c, "foo", {
  value: "foobar",
  enumerable: true,
  configurable: true,
  writable: true,
});

assert.sameValue(c.bar, "barbaz");
assert(
  !Object.prototype.hasOwnProperty.call(C, "bar"),
  "bar doesn't appear as an own property on the C constructor"
);
assert(
  !Object.prototype.hasOwnProperty.call(C.prototype, "bar"),
  "bar doesn't appear as an own property on the C prototype"
);

verifyProperty(c, "bar", {
  value: "barbaz",
  enumerable: true,
  configurable: true,
  writable: true,
});

assert.sameValue(c.o(1), 1);
assert.sameValue(c.℘(1), 1);
assert.sameValue(c.ZW_‌_NJ(1), 1);
assert.sameValue(c.ZW_‍_J(1), 1);
