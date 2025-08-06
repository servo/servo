/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://heycam.github.io/webidl/#quotaexceedederror

[Exposed=*, Serializable]
interface QuotaExceededError : DOMException {
  [Throws] constructor(optional DOMString message = "", optional QuotaExceededErrorOptions options = {});

  readonly attribute double? quota;
  readonly attribute double? requested;
};

dictionary QuotaExceededErrorOptions {
  double quota;
  double requested;
};
