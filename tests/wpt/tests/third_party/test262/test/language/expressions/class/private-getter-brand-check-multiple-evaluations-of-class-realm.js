// Copyright (C) 2019 Jaideep Bhoosreddy (Bloomberg LP). All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Every new evaluation of a class creates a different brand (private getter)
esid: sec-privatefieldget
info: |
  ClassTail : ClassHeritage { ClassBody }
    ...
    11. Let proto be ObjectCreate(protoParent).
    ...
    31. If PrivateBoundIdentifiers of ClassBody contains a Private Name P such that the P's [[Kind]] field is either "method" or "accessor",
      a. Set F.[[PrivateBrand]] to proto.
    ...

  PrivateBrandCheck(O, P)
    1. If O.[[PrivateBrands]] does not contain an entry e such that SameValue(e, P.[[Brand]]) is true,
      a. Throw a TypeError exception.
features: [class, class-methods-private, cross-realm]
flags: [noStrict]
---*/

let realm1 = $262.createRealm();
let realm2 = $262.createRealm();
let eval1 = realm1.global.eval;
let eval2 = realm2.global.eval;

let classStringExpression = `(
class {
  get #m() { return 'test262'; }

  access(o) {
    return o.#m;
  }
}
)`;

let createAndInstantiateClass = function (_eval) {
  return new (_eval(classStringExpression));
};

let c1 = createAndInstantiateClass(eval1);
let c2 = createAndInstantiateClass(eval2);

assert.sameValue(c1.access(c1), 'test262');
assert.sameValue(c2.access(c2), 'test262');

assert.throws(realm1.global.TypeError, function() {
  c1.access(c2);
}, 'invalid access of c1 private method');

assert.throws(realm2.global.TypeError, function() {
  c2.access(c1);
}, 'invalid access of c2 private method');
