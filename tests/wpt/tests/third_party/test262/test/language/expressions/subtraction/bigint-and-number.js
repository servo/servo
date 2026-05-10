// Copyright (C) 2018 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-subtraction-operator-minus-runtime-semantics-evaluation
description: Mixing BigInt and Number produces a TypeError for subtraction operator
features: [BigInt]
info: |
  Let lnum be ? ToNumeric(leftValue).
  Let rnum be ? ToNumeric(rightValue).
  If Type(lnum) does not equal Type(rnum), throw a TypeError exception.
---*/
assert.throws(TypeError, function() {
  1n - 1;
}, '1n - 1 throws TypeError');

assert.throws(TypeError, function() {
  1 - 1n;
}, '1 - 1n throws TypeError');

assert.throws(TypeError, function() {
  Object(1n) - 1;
}, 'Object(1n) - 1 throws TypeError');

assert.throws(TypeError, function() {
  1 - Object(1n);
}, '1 - Object(1n) throws TypeError');

assert.throws(TypeError, function() {
  1n - Object(1);
}, '1n - Object(1) throws TypeError');

assert.throws(TypeError, function() {
  Object(1) - 1n;
}, 'Object(1) - 1n throws TypeError');

assert.throws(TypeError, function() {
  Object(1n) - Object(1);
}, 'Object(1n) - Object(1) throws TypeError');

assert.throws(TypeError, function() {
  Object(1) - Object(1n);
}, 'Object(1) - Object(1n) throws TypeError');

assert.throws(TypeError, function() {
  1n - NaN;
}, '1n - NaN throws TypeError');

assert.throws(TypeError, function() {
  NaN - 1n;
}, 'NaN - 1n throws TypeError');

assert.throws(TypeError, function() {
  1n - Infinity;
}, '1n - Infinity throws TypeError');

assert.throws(TypeError, function() {
  Infinity - 1n;
}, 'Infinity - 1n throws TypeError');

assert.throws(TypeError, function() {
  1n - true;
}, '1n - true throws TypeError');

assert.throws(TypeError, function() {
  true - 1n;
}, 'true - 1n throws TypeError');

assert.throws(TypeError, function() {
  1n - '1';
}, '1n - "1" throws TypeError');

assert.throws(TypeError, function() {
  '1' - 1n;
}, '"1" - 1n throws TypeError');

assert.throws(TypeError, function() {
  1n - null;
}, '1n - null throws TypeError');

assert.throws(TypeError, function() {
  null - 1n;
}, 'null - 1n throws TypeError');

assert.throws(TypeError, function() {
  1n - undefined;
}, '1n - undefined throws TypeError');

assert.throws(TypeError, function() {
  undefined - 1n;
}, 'undefined - 1n throws TypeError');
