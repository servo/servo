/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://html.spec.whatwg.org/multipage/#the-origin-interface
[Exposed=*]
interface Origin {
  constructor();

  [Throws] static Origin from(any value);

  readonly attribute boolean opaque;

  boolean isSameOrigin(Origin other);
  boolean isSameSite(Origin other);
};
