// This file was procedurally generated from the following sources:
// - src/class-elements/static-private-getter.case
// - src/class-elements/default/cls-expr.template
/*---
description: static private getter declaration and usage (field definitions in a class expression)
esid: prod-FieldDefinition
features: [class-static-methods-private, class]
flags: [generated]
info: |
    MethodDefinition :
      get ClassElementName () { FunctionBody }
      set ClassElementName ( PropertySetParameterList ) { FunctionBody }

    ClassTail : ClassHeritage { ClassBody }
      ...
      33. If PrivateBoundIdentifiers of ClassBody contains a Private Name P such that P's [[Kind]] field is either "method" or "accessor" and P's [[Brand]] is F,
        a. PrivateBrandAdd(F, F).
      34. For each item fieldRecord in order from staticFields,
        a. Perform ? DefineField(F, field).

    PrivateFieldGet (P, O)
      1. Assert: P is a Private Name.
      2. If O is not an object, throw a TypeError exception.
      3. If P.[[Kind]] is "field",
      ...
      4. Perform ? PrivateBrandCheck(O, P).
      5. If P.[[Kind]] is "method",
      ...
      6. Else,
        a. Assert: P.[[Kind]] is "accessor".
        b. If P does not have a [[Get]] field, throw a TypeError exception.
        c. Let getter be P.[[Get]].
        d. Return ? Call(getter, O).

    PrivateBrandCheck(O, P)
      1. If O.[[PrivateBrands]] does not contain an entry e such that SameValue(e, P.[[Brand]]) is true,
        a. Throw a TypeError exception.

---*/


var C = class {
  static get #f() {
    return 'Test262';
  }

  static access() {
    return this.#f;
  }
}

assert.sameValue(C.access(), 'Test262');
assert.throws(TypeError, function() {
  C.access.call({});
}, 'Accessed static private getter from an arbitrary object');
