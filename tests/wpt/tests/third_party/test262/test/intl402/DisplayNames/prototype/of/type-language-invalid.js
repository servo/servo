// Copyright (C) 2023 Igalia S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-Intl.DisplayNames.prototype.of
description: Throws a RangeError for invalid `language` codes
info: |
  12.3.3 Intl.DisplayNames.prototype.of ( code )

  1. If type is "language", then
      a. If code cannot be matched by the unicode_language_id Unicode locale nonterminal, throw a RangeError exception.
      b. If IsStructurallyValidLanguageTag(code) is false, throw a RangeError exception.
      c. Return CanonicalizeUnicodeLocaleId(code).
features: [Intl.DisplayNames-v2]
---*/

var displayNames = new Intl.DisplayNames(undefined, {type: 'language'});

assert.throws(RangeError, function() {
  displayNames.of('');
}, 'invalid language subtag - empty string');

assert.throws(RangeError, function() {
  displayNames.of('a');
}, 'invalid language subtag - only one character');

assert.throws(RangeError, function() {
  displayNames.of('abcdefghi');
}, 'invalid language subtag - greater than 8 characters');

assert.throws(RangeError, function() {
  displayNames.of('en-u-hebrew');
}, 'singleton subtag');

assert.throws(RangeError, function() {
  displayNames.of('aa-aaaa-bbbb');
}, 'multiple script subtags');

assert.throws(RangeError, function() {
  displayNames.of('aa-aaaaa-aaaaa');
}, 'duplicate variant subtag');

assert.throws(RangeError, function() {
  displayNames.of('aa-bb-cc');
}, 'multiple region subtags');

assert.throws(RangeError, function() {
  displayNames.of('1a');
}, 'invalid language subtag - leading digit');

assert.throws(RangeError, function() {
  displayNames.of('aa-1a');
}, 'leading-digit subtag of length 2');

assert.throws(RangeError, function() {
  displayNames.of('aa-1aa');
}, 'leading-digit non-numeric subtag of length 3');

assert.throws(RangeError, function() {
  displayNames.of('@#$%@#$');
}, 'invalid characters');

assert.throws(RangeError, function() {
  displayNames.of('en-US-');
}, 'separator not followed by subtag');

assert.throws(RangeError, function() {
  displayNames.of('-en');
}, 'separator at start');

assert.throws(RangeError, function() {
  displayNames.of('en--GB');
}, 'missing subtag between separators');

assert.throws(RangeError, function() {
  displayNames.of('root');
}, 'BCP 47-incompatible CLDR syntax ("root" instead of "und")');

assert.throws(RangeError, function(){
  displayNames.of('abcd-GB');
}, 'BCP 47-incompatible CLDR syntax (script subtag without a language subtag)');

assert.throws(RangeError, function(){
  displayNames.of('abcd');
}, 'BCP 47-incompatible CLDR syntax (bare script subtag)');

assert.throws(RangeError, function(){
  displayNames.of('en_GB');
}, 'BCP 47-incompatible CLDR syntax (_ as separator)');
