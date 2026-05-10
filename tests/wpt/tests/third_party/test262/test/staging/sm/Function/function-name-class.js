// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  Anonymous class with name method shouldn't be affected by assignment
info: bugzilla.mozilla.org/show_bug.cgi?id=883377
esid: pending
---*/

var classWithStaticNameMethod = class { static name() {} };
assert.sameValue(typeof classWithStaticNameMethod.name, "function");

var classWithStaticNameGetter = class { static get name() { return "static name"; } };
assert.sameValue(typeof Object.getOwnPropertyDescriptor(classWithStaticNameGetter, "name").get, "function");
assert.sameValue(classWithStaticNameGetter.name, "static name");

var classWithStaticNameSetter = class { static set name(v) {} };
assert.sameValue(typeof Object.getOwnPropertyDescriptor(classWithStaticNameSetter, "name").set, "function");

var n = "NAME".toLowerCase();
var classWithStaticNameMethodComputed = class { static [n]() {} };
assert.sameValue(typeof classWithStaticNameMethodComputed.name, "function");

// It doesn't apply for non-static method.

var classWithNameMethod = class { name() {} };
assert.sameValue(classWithNameMethod.name, "classWithNameMethod");

var classWithNameGetter = class { get name() { return "name"; } };
assert.sameValue(classWithNameGetter.name, "classWithNameGetter");

var classWithNameSetter = class { set name(v) {} };
assert.sameValue(classWithNameSetter.name, "classWithNameSetter");
