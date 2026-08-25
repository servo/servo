// Copyright 2024 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.DurationFormat
description: Verifies that default style settings for units following units with "numeric" or "2-digit" style are honored.
info: |
    GetDurationUnitOptions (unit, options, baseStyle, stylesList, digitalBase, prevStyle)
      (...)
      3. If style is undefined, then
        (...)
        i. If prevStyle is "fractional", "numeric" or "2-digit", then
          (...)
          2. Set style to "numeric".
      (...)
      9. If prevStyle is "numeric" or "2-digit", then
        (...)
        b. If unit is "minutes" or "seconds", then
                i. Set style to "2-digit".
features: [Intl.DurationFormat]
---*/

for (const numericLikeStyle of ["numeric", "2-digit"]){
  var opts = new Intl.DurationFormat([], {hours: numericLikeStyle}).resolvedOptions();

  assert.sameValue(opts.minutes, "2-digit", `minutes default value should be '2-digit' when following any ${numericLikeStyle}-styled unit`);
  assert.sameValue(opts.seconds, "2-digit", `seconds default value should be '2-digit' when following any ${numericLikeStyle}-styled unit`);
  assert.sameValue(opts.milliseconds, "numeric", `milliseconds default value should be 'numeric' when following any ${numericLikeStyle}-styled unit`);
  assert.sameValue(opts.microseconds, "numeric", `microseconds default value should be 'numeric' when following any ${numericLikeStyle}-styled unit`);
  assert.sameValue(opts.nanoseconds, "numeric", `nanoseconds default value should be 'numeric' when following any ${numericLikeStyle}-styled unit`);
}
