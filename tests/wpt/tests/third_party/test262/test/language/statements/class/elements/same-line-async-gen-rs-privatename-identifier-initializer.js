// This file was procedurally generated from the following sources:
// - src/class-elements/rs-privatename-identifier-initializer.case
// - src/class-elements/productions/cls-decl-after-same-line-async-gen.template
/*---
description: Valid PrivateName (field definitions after an async generator in the same line)
esid: prod-FieldDefinition
features: [class-fields-private, class, class-fields-public, async-iteration]
flags: [generated, async]
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

---*/


class C {
  async *m() { return 42; } #$ = 1; #_ = 1; #\u{6F} = 1; #\u2118 = 1; #ZW_\u200C_NJ = 1; #ZW_\u200D_J = 1;
  $() {
    return this.#$;
  }
  _() {
    return this.#_;
  }
  \u{6F}() {
    return this.#\u{6F};
  }
  \u2118() {
    return this.#\u2118;
  }
  ZW_\u200C_NJ() {
    return this.#ZW_\u200C_NJ;
  }
  ZW_\u200D_J() {
    return this.#ZW_\u200D_J;
  }
}

var c = new C();

assert(
  !Object.prototype.hasOwnProperty.call(c, "m"),
  "m doesn't appear as an own property on the C instance"
);
assert.sameValue(c.m, C.prototype.m);

verifyProperty(C.prototype, "m", {
  enumerable: false,
  configurable: true,
  writable: true,
}, {restore: true});

c.m().next().then(function(v) {
  assert.sameValue(v.value, 42);
  assert.sameValue(v.done, true);

  function assertions() {
    // Cover $DONE handler for async cases.
    function $DONE(error) {
      if (error) {
        throw new Test262Error('Test262:AsyncTestFailure')
      }
    }
    assert.sameValue(c.$(), 1);
    assert.sameValue(c._(), 1);
    assert.sameValue(c.\u{6F}(), 1);
    assert.sameValue(c.\u2118(), 1);
    assert.sameValue(c.ZW_\u200C_NJ(), 1);
    assert.sameValue(c.ZW_\u200D_J(), 1);

  }

  return Promise.resolve(assertions());
}).then($DONE, $DONE);
