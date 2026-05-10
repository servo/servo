// Copyright (C) 2024 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration
description: >
  ToNumber conversion throws.
info: |
  Temporal.Duration ( [ years [ , months [ , weeks [ , days [ , hours [ ,
                      minutes [ , seconds [ , milliseconds [ , microseconds [ ,
                      nanoseconds ] ] ] ] ] ] ] ] ] ] )

  ...
  2. If years is undefined, let y be 0; else let y be ? ToIntegerIfIntegral(years).
  3. If months is undefined, let mo be 0; else let mo be ? ToIntegerIfIntegral(months).
  4. If weeks is undefined, let w be 0; else let w be ? ToIntegerIfIntegral(weeks).
  5. If days is undefined, let d be 0; else let d be ? ToIntegerIfIntegral(days).
  6. If hours is undefined, let h be 0; else let h be ? ToIntegerIfIntegral(hours).
  7. If minutes is undefined, let m be 0; else let m be ? ToIntegerIfIntegral(minutes).
  8. If seconds is undefined, let s be 0; else let s be ? ToIntegerIfIntegral(seconds).
  9. If milliseconds is undefined, let ms be 0; else let ms be ? ToIntegerIfIntegral(milliseconds).
  10. If microseconds is undefined, let mis be 0; else let mis be ? ToIntegerIfIntegral(microseconds).
  11. If nanoseconds is undefined, let ns be 0; else let ns be ? ToIntegerIfIntegral(nanoseconds).
  ...

  ToIntegerIfIntegral ( argument )

  1. Let number be ? ToNumber(argument).
  ...
features: [Temporal]
---*/

for (var invalid of [Symbol(), 0n]) {
  assert.throws(TypeError, () => new Temporal.Duration(invalid));
  assert.throws(TypeError, () => new Temporal.Duration(0, invalid));
  assert.throws(TypeError, () => new Temporal.Duration(0, 0, invalid));
  assert.throws(TypeError, () => new Temporal.Duration(0, 0, 0, invalid));
  assert.throws(TypeError, () => new Temporal.Duration(0, 0, 0, 0, invalid));
  assert.throws(TypeError, () => new Temporal.Duration(0, 0, 0, 0, 0, invalid));
  assert.throws(TypeError, () => new Temporal.Duration(0, 0, 0, 0, 0, 0, invalid));
  assert.throws(TypeError, () => new Temporal.Duration(0, 0, 0, 0, 0, 0, 0, invalid));
  assert.throws(TypeError, () => new Temporal.Duration(0, 0, 0, 0, 0, 0, 0, 0, invalid));
  assert.throws(TypeError, () => new Temporal.Duration(0, 0, 0, 0, 0, 0, 0, 0, 0, invalid));
}
