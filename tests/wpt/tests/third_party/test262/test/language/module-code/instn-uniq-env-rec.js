// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: Modules have distinct environment records
esid: sec-moduledeclarationinstantiation
info: |
    [...]
    6. Let env be NewModuleEnvironment(realm.[[GlobalEnv]]).
    7. Set module.[[Environment]] to env.
    [...]

    8.1.2.6 NewModuleEnvironment (E)

    1. Let env be a new Lexical Environment.
    [...]
flags: [module]
features: [generators]
---*/

import './instn-uniq-env-rec-other_FIXTURE.js'
var first = 1;
let second = 2;
const third = 3;
class fourth {}
function fifth() { return 'fifth'; }
function* sixth() { return 'sixth'; }

assert.sameValue(first, 1);
assert.sameValue(second, 2);
assert.sameValue(third, 3);
assert.sameValue(typeof fourth, 'function', 'class declaration');
assert.sameValue(typeof fifth, 'function', 'function declaration');
assert.sameValue(fifth(), 'fifth');
assert.sameValue(typeof sixth, 'function', 'generator function declaration');
assert.sameValue(sixth().next().value, 'sixth');

// Two separate mechanisms are required to ensure that no binding has been
// created for a given identifier. A "bare" reference should produce a
// ReferenceError for non-existent bindings and uninitialized bindings. A
// reference through the `typeof` operator should succeed for non-existent
// bindings and initialized bindings.  Only non-existent bindings satisfy both
// tests.
typeof seventh;
assert.throws(ReferenceError, function() {
  seventh;
});

typeof eight;
assert.throws(ReferenceError, function() {
  eighth;
});

typeof ninth;
assert.throws(ReferenceError, function() {
  ninth;
});

typeof tenth;
assert.throws(ReferenceError, function() {
  tenth;
});

typeof eleventh;
assert.throws(ReferenceError, function() {
  eleventh;
});

typeof twelfth;
assert.throws(ReferenceError, function() {
  twelfth;
});
