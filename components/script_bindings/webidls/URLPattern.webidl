/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://urlpattern.spec.whatwg.org/#urlpattern

typedef /* USVString or */ URLPatternInit    URLPatternInput;

[Exposed=(Window,Worker), Pref="dom_urlpattern_enabled"]
interface URLPattern {
  // [Throws] constructor(URLPatternInput input, USVString baseURL, optional URLPatternOptions options = {});
  [Throws] constructor(optional URLPatternInput input = {}, optional URLPatternOptions options = {});

//   boolean test(optional URLPatternInput input = {}, optional USVString baseURL);

//   URLPatternResult? exec(optional URLPatternInput input = {}, optional USVString baseURL);

  readonly attribute USVString protocol;
  readonly attribute USVString username;
  readonly attribute USVString password;
  readonly attribute USVString hostname;
  readonly attribute USVString port;
  readonly attribute USVString pathname;
  readonly attribute USVString search;
  readonly attribute USVString hash;

  readonly attribute boolean hasRegExpGroups;
};

dictionary URLPatternInit {
  USVString protocol;
  USVString username;
  USVString password;
  USVString hostname;
  USVString port;
  USVString pathname;
  USVString search;
  USVString hash;
  USVString baseURL;
};

dictionary URLPatternOptions {
  boolean ignoreCase = false;
};

// dictionary URLPatternResult {
//   sequence<URLPatternInput> inputs;

//   URLPatternComponentResult protocol;
//   URLPatternComponentResult username;
//   URLPatternComponentResult password;
//   URLPatternComponentResult hostname;
//   URLPatternComponentResult port;
//   URLPatternComponentResult pathname;
//   URLPatternComponentResult search;
//   URLPatternComponentResult hash;
// };

// dictionary URLPatternComponentResult {
//   USVString input;
//   record<USVString, (USVString or undefined)> groups;
// };
