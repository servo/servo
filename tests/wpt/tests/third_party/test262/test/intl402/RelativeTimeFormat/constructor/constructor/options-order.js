// Copyright 2018 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.RelativeTimeFormat
description: Checks the order of operations on the options argument to the RelativeTimeFormat constructor.
info: |
    InitializeRelativeTimeFormat (relativeTimeFormat, locales, options)
    7. Let matcher be ? GetOption(options, "localeMatcher", "string", «"lookup", "best fit"», "best fit").
    14. Let s be ? GetOption(options, "style", "string", «"long", "short", "narrow"», "long").
    16. Let numeric be ? GetOption(options, "numeric", "string", «"always", "auto"», "always").
includes: [compareArray.js]
features: [Intl.RelativeTimeFormat]
---*/

const callOrder = [];

new Intl.RelativeTimeFormat([], {
  get localeMatcher() {
    callOrder.push("localeMatcher");
    return {
      toString() {
        callOrder.push("localeMatcher toString");
        return "best fit";
      }
    };
  },
  get style() {
    callOrder.push("style");
    return {
      toString() {
        callOrder.push("style toString");
        return "long";
      }
    };
  },
  get numberingSystem() {
    callOrder.push("numberingSystem");
    return {
      toString() {
        callOrder.push("numberingSystem toString");
        return "abc";
      }
    };
  },
  get numeric() {
    callOrder.push("numeric");
    return {
      toString() {
        callOrder.push("numeric toString");
        return "always";
      }
    };
  },
});

assert.compareArray(callOrder, [
  "localeMatcher",
  "localeMatcher toString",
  "numberingSystem",
  "numberingSystem toString",
  "style",
  "style toString",
  "numeric",
  "numeric toString",
]);
