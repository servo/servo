// Copyright (C) 2019 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.DisplayNames.prototype.resolvedOptions
description: >
  Throws a TypeError if this does not have an [[InitializedDisplayNames]] internal slot.
info: |
  Intl.DisplayNames.prototype.resolvedOptions ()

  1. Let pr be the this value.
  2. If Type(pr) is not Object or pr does not have an [[InitializedDisplayNames]] internal slot,
    throw a TypeError exception.
  ...
features: [Intl.DisplayNames]
---*/

var resolvedOptions = Intl.DisplayNames.prototype.resolvedOptions;

assert.throws(TypeError, function() {
  Intl.DisplayNames.prototype.resolvedOptions();
}, 'Intl.DisplayNames.prototype does not have the internal slot');

assert.throws(TypeError, function() {
  resolvedOptions.call({});
}, 'ordinary object');

assert.throws(TypeError, function() {
  resolvedOptions.call(Intl.DisplayNames);
}, 'Intl.DisplayNames does not have the internal slot');

assert.throws(TypeError, function() {
  resolvedOptions.call(Intl);
}, 'Intl does not have the internal slot');

// Not DisplayNames!!!
var dtf = new Intl.DateTimeFormat();

assert.throws(TypeError, function() {
  resolvedOptions.call(dtf);
}, 'resolvedOptions cannot be used with instances from different Intl ctors');
