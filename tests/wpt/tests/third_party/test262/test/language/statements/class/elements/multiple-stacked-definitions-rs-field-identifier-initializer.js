// This file was procedurally generated from the following sources:
// - src/class-elements/rs-field-identifier-initializer.case
// - src/class-elements/productions/cls-decl-multiple-stacked-definitions.template
/*---
description: Valid FieldDefinition (multiple stacked fields definitions through ASI)
esid: prod-FieldDefinition
features: [class-fields-public, class]
flags: [generated]
includes: [propertyHelper.js]
info: |
    ClassElement :
      ...
      FieldDefinition ;
      ;

    FieldDefinition :
      ClassElementName Initializer _opt

    ClassElementName :
      PropertyName

    PropertyName :
      LiteralPropertyName
      ComputedPropertyName

    LiteralPropertyName :
      IdentifierName

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
  $ = 1; _ = 1; \u{6F} = 1; \u2118 = 1; ZW_\u200C_NJ = 1; ZW_\u200D_J = 1
  foo = "foobar"
  bar = "barbaz";
  
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

assert.sameValue(c.$, 1);
assert.sameValue(c._, 1);
assert.sameValue(c.\u{6F}, 1);
assert.sameValue(c.\u2118, 1);
assert.sameValue(c.ZW_\u200C_NJ, 1);
assert.sameValue(c.ZW_\u200D_J, 1);
