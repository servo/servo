/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
/*
 * The origin of this IDL file is
 * https://html.spec.whatwg.org/multipage/#imagedata
 *
 * © Copyright 2004-2011 Apple Computer, Inc., Mozilla Foundation, and Opera Software ASA.
 * You are granted a license to use, reproduce and create derivative works of this document.
 */

[Exposed=(Window,Worker)]
interface ImageData {
  [Throws] constructor(unsigned long sw, unsigned long sh);
  [Throws] constructor(/* Uint8ClampedArray */ object data, unsigned long sw, optional unsigned long sh);
  //[Constant]
  readonly attribute unsigned long width;
  //[Constant]
  readonly attribute unsigned long height;
  //[Constant, StoreInSlot]
  readonly attribute Uint8ClampedArray data;
};
