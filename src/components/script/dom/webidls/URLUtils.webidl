/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// http://url.spec.whatwg.org/#urlutils
[NoInterfaceObject]
interface URLUtils {
  //stringifier attribute ScalarValueString href;
  readonly attribute DOMString href;
  //readonly attribute ScalarValueString origin;

  //         attribute ScalarValueString protocol;
  //         attribute ScalarValueString username;
  //         attribute ScalarValueString password;
  //         attribute ScalarValueString host;
  //         attribute ScalarValueString hostname;
  //         attribute ScalarValueString port;
  //         attribute ScalarValueString pathname;
  //         attribute ScalarValueString search;
  readonly attribute DOMString search;
  //         attribute URLSearchParams searchParams;
  //         attribute ScalarValueString hash;
  readonly attribute DOMString hash;
};
