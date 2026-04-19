// This file was procedurally generated from the following sources:
// - src/class-elements/static-private-method-and-instance-method-brand-check.case
// - src/class-elements/default/cls-expr.template
/*---
description: Brand for static private names and instance private names are different (field definitions in a class expression)
esid: prod-FieldDefinition
features: [class-static-methods-private, class-methods-private, class]
flags: [generated]
info: |
    ClassTail : ClassHeritage { ClassBody }
      ...
      32. If PrivateBoundIdentifiers of ClassBody contains a Private Name P such that P's [[Kind]] field is either "method" or "accessor" and P's [[Brand]] field is proto,
        a. Set F.[[PrivateBrand]] to proto.
      33. If PrivateBoundIdentifiers of ClassBody contains a Private Name P such that P's [[Kind]] field is either "method" or "accessor" and P's [[Brand]] is F,
        a. PrivateBrandAdd(F, F).
      ...

    PrivateFieldGet (P, O)
      1. Assert: P is a Private Name.
      2. If O is not an object, throw a TypeError exception.
      3. If P.[[Kind]] is "field",
      ...
      4. Perform ? PrivateBrandCheck(O, P).
      5. If P.[[Kind]] is "method",
        a. Return P.[[Value]].
      ...

    PrivateBrandCheck(O, P)
      1. If O.[[PrivateBrands]] does not contain an entry e such that SameValue(e, P.[[Brand]]) is true,
        a. Throw a TypeError exception.

---*/


var C = class {
  static #f() {
    return 'static';
  }

  static access() {
    return this.#f();
  }

  #instanceMethod() {
    return 'instance';
  }

  instanceAccess() {
    return this.#instanceMethod();
  }
}

let c = new C();
assert.sameValue(C.access(), 'static');
assert.sameValue(c.instanceAccess(), 'instance');

assert.throws(TypeError, function() {
  C.access.call(c);
}, 'Accessed static private method from instance of C');

assert.throws(TypeError, function() {
  c.instanceAccess.call(C);
}, 'Accessed instance private method from C');

