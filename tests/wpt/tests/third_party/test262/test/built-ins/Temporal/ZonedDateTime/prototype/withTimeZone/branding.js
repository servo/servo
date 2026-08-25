// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.withtimezone
description: Throw a TypeError if the receiver is invalid
features: [Symbol, Temporal]
---*/

const withTimeZone = Temporal.ZonedDateTime.prototype.withTimeZone;

assert.sameValue(typeof withTimeZone, "function");

const args = ["UTC"];

assert.throws(TypeError, () => withTimeZone.apply(undefined, args), "undefined");
assert.throws(TypeError, () => withTimeZone.apply(null, args), "null");
assert.throws(TypeError, () => withTimeZone.apply(true, args), "true");
assert.throws(TypeError, () => withTimeZone.apply("", args), "empty string");
assert.throws(TypeError, () => withTimeZone.apply(Symbol(), args), "symbol");
assert.throws(TypeError, () => withTimeZone.apply(1, args), "1");
assert.throws(TypeError, () => withTimeZone.apply({}, args), "plain object");
assert.throws(TypeError, () => withTimeZone.apply(Temporal.ZonedDateTime, args), "Temporal.ZonedDateTime");
assert.throws(TypeError, () => withTimeZone.apply(Temporal.ZonedDateTime.prototype, args), "Temporal.ZonedDateTime.prototype");
