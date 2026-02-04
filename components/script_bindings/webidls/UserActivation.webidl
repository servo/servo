/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://html.spec.whatwg.org/multipage/#the-useractivation-interface
[Exposed=Window]
interface UserActivation {
  readonly attribute boolean hasBeenActive;
  readonly attribute boolean isActive;
};

partial interface Navigator {
  [SameObject] readonly attribute UserActivation userActivation;
};
