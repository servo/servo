// This file was procedurally generated from the following sources:
// - src/class-elements/rs-static-async-generator-method-privatename-identifier-alt.case
// - src/class-elements/productions/cls-decl-new-sc-line-generator.template
/*---
description: Valid Static AsyncGeneratorMethod PrivateName (field definitions followed by a method in a new line with a semicolon)
esid: prod-FieldDefinition
features: [class-static-methods-private, class, class-fields-public, generators]
flags: [generated, async]
includes: [propertyHelper.js]
info: |
    ClassElement :
      MethodDefinition
      static MethodDefinition
      FieldDefinition ;
      static FieldDefinition ;
      ;

    MethodDefinition :
      AsyncGeneratorMethod

    AsyncGeneratorMethod :
      async [no LineTerminator here] * ClassElementName ( UniqueFormalParameters){ AsyncGeneratorBody }

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

---*/


class C {
  static async * #$(value) {
    yield * await value;
  }
  static async * #_(value) {
    yield * await value;
  }
  static async * #o(value) {
    yield * await value;
  }
  static async * #℘(value) {
    yield * await value;
  }
  static async * #ZW_‌_NJ(value) {
    yield * await value;
  }
  static async * #ZW_‍_J(value) {
    yield * await value;
  };
  *m() { return 42; }
  static get $() {
   return this.#$;
  }
  static get _() {
   return this.#_;
  }
  static get o() {
   return this.#o;
  }
  static get ℘() { // DO NOT CHANGE THE NAME OF THIS FIELD
   return this.#℘;
  }
  static get ZW_‌_NJ() { // DO NOT CHANGE THE NAME OF THIS FIELD
   return this.#ZW_‌_NJ;
  }
  static get ZW_‍_J() { // DO NOT CHANGE THE NAME OF THIS FIELD
   return this.#ZW_‍_J;
  }

}

var c = new C();

assert.sameValue(c.m().next().value, 42);
assert.sameValue(c.m, C.prototype.m);
assert(
  !Object.prototype.hasOwnProperty.call(c, "m"),
  "m doesn't appear as an own property on the C instance"
);

verifyProperty(C.prototype, "m", {
  enumerable: false,
  configurable: true,
  writable: true,
});

Promise.all([
  C.$([1]).next(),
  C._([1]).next(),
  C.o([1]).next(),
  C.℘([1]).next(), // DO NOT CHANGE THE NAME OF THIS FIELD
  C.ZW_‌_NJ([1]).next(), // DO NOT CHANGE THE NAME OF THIS FIELD
  C.ZW_‍_J([1]).next(), // DO NOT CHANGE THE NAME OF THIS FIELD
]).then(results => {

  assert.sameValue(results[0].value, 1);
  assert.sameValue(results[1].value, 1);
  assert.sameValue(results[2].value, 1);
  assert.sameValue(results[3].value, 1);
  assert.sameValue(results[4].value, 1);
  assert.sameValue(results[5].value, 1);

}).then($DONE, $DONE);
