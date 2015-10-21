/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// https://html.spec.whatwg.org/multipage/#location
/*[Unforgeable]*/ interface Location {
  /*stringifier*/ attribute USVString href;
  //         attribute USVString origin;
           attribute USVString protocol;
           attribute USVString host;
           attribute USVString hostname;
           attribute USVString port;
           attribute USVString pathname;
           attribute USVString search;
           attribute USVString hash;

  void assign(USVString url);
  //void replace(USVString url);
  void reload();

  //[SameObject] readonly attribute USVString[] ancestorOrigins;

  // This is only doing as well as gecko right now.
  // https://github.com/servo/servo/issues/7590 is on file for
  // adding attribute stringifier support.
  stringifier;
};
