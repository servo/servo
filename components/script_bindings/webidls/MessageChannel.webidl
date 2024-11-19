/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
/*
 * The origin of this IDL file is:
 * https://html.spec.whatwg.org/multipage/#messagechannel
 */

[Exposed=(Window,Worker)]
interface MessageChannel {
  constructor();
  readonly attribute MessagePort port1;
  readonly attribute MessagePort port2;
};
