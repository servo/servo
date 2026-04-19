// Copyright (C) 2019 Caio Lima (Igalia SL). All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Every new evaluation of a class creates a different Private Name (private static getter)
esid: sec-runtime-semantics-evaluate-name
info: |
  ClassTail : ClassHeritage { ClassBody }
    ...
    19. Let F be constructorInfo.[[Closure]].
    20. If ClassHeritage_opt is present and protoParent is not null, then set F.[[ConstructorKind]] to "derived".
    21. Perform MakeConstructor(F, false, proto).
    22. Perform MakeClassConstructor(F).
    ...
    33. If PrivateBoundIdentifiers of ClassBody contains a Private Name P such that P's [[Kind]] field is either "method" or "accessor" and P's [[Brand]] is F,
      a. PrivateBrandAdd(F, F).
    ...

  PrivateBrandCheck(O, P)
    1. If O.[[PrivateBrands]] does not contain an entry e such that SameValue(e, P.[[Brand]]) is true,
      a. Throw a TypeError exception.
features: [class, class-static-methods-private, cross-realm]
flags: [noStrict]
---*/

let global1 = $262.createRealm().global;
let global2 = $262.createRealm().global;
let eval1 = global1.eval;
let eval2 = global2.eval;

let classStringExpression = `(
class {
  static get #m() {
    return 'test262';
  };

  static access() {
    return this.#m;
  }
}
)`;

let evalClass = function (_eval) {
  return _eval(classStringExpression);
};

let C1 = evalClass(eval1);
let C2 = evalClass(eval2);

assert.sameValue(C1.access(), 'test262');
assert.sameValue(C2.access(), 'test262');

assert.throws(global1.TypeError, function() {
  C1.access.call(C2);
}, 'invalid access of C1 private static getter');

assert.throws(global2.TypeError, function() {
  C2.access.call(C1);
}, 'invalid access of C2 private static getter');
