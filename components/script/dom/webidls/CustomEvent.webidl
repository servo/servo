/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
/*
 * For more information on this interface please see
 * https://dom.spec.whatwg.org/#interface-customevent
 *
 * To the extent possible under law, the editors have waived
 * all copyright and related or neighboring rights to this work.
 * In addition, as of 1 May 2014, the editors have made this specification
 * available under the Open Web Foundation Agreement Version 1.0,
 * which is available at
 * http://www.openwebfoundation.org/legal/the-owf-1-0-agreements/owfa-1-0.
 */

[Constructor(DOMString type, optional CustomEventInit eventInitDict),
 Exposed=(Window,Worker)]
interface CustomEvent : Event {
  readonly attribute any detail;

  void initCustomEvent(DOMString type, boolean bubbles, boolean cancelable, any detail);
};

dictionary CustomEventInit : EventInit {
  any detail = null;
};
