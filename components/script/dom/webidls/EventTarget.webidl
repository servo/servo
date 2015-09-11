/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
/*
 * https://dom.spec.whatwg.org/#interface-eventtarget
 */

[Abstract]
interface EventTarget {
  void addEventListener(DOMString type,
                        EventListener? listener,
                        optional boolean capture = false);
  void removeEventListener(DOMString type,
                           EventListener? listener,
                           optional boolean capture = false);
  [Throws]
  boolean dispatchEvent(Event event);
};
