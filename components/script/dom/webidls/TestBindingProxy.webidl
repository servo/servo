/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
/*
 * This IDL file was created to test the special operations (see
 * https://heycam.github.io/webidl/#idl-special-operations) without converting
 * TestBinding.webidl into a proxy.
 *
 */

// This interface is entirely internal to Servo, and should not be accessible to
// web pages.

interface TestBindingProxy : TestBinding {
  readonly attribute unsigned long length;

  getter DOMString getNamedItem(DOMString name);

  setter creator void setNamedItem(DOMString name, DOMString value);

  getter DOMString getItem(unsigned long index);

  setter creator void setItem(unsigned long index, DOMString value);

  deleter void removeItem(DOMString name);

  stringifier;
};
