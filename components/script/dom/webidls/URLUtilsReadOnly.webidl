/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// http://url.spec.whatwg.org/#urlutilsreadonly
[NoInterfaceObject/*,
 Exposed=(Window,Worker)*/]
interface URLUtilsReadOnly {
  //stringifier readonly attribute ScalarValueString href;
  readonly attribute DOMString href;
  //readonly attribute ScalarValueString origin;

  //readonly attribute ScalarValueString protocol;
  //readonly attribute ScalarValueString host;
  //readonly attribute ScalarValueString hostname;
  //readonly attribute ScalarValueString port;
  //readonly attribute ScalarValueString pathname;
  //readonly attribute ScalarValueString search;
  readonly attribute DOMString search;
  //readonly attribute ScalarValueString hash;
  readonly attribute DOMString hash;
};
