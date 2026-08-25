// Copyright (C) 2020 ExE Boss. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-regexp.prototype.compile
description: RegExp.prototype.compile throws a TypeError for calls on subclasses
features: [legacy-regexp,class]
---*/

const subclass_regexp = new (class extends RegExp {})("");

assert.throws(
  TypeError,
  function () {
    subclass_regexp.compile();
  });

assert.throws(
  TypeError,
  function () {
    RegExp.prototype.compile.call(subclass_regexp);
  });
