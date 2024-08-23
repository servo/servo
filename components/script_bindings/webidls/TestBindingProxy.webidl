/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
/*
 * This IDL file was created to test the special operations (see
 * https://heycam.github.io/webidl/#idl-special-operations) without converting
 * TestBinding.webidl into a proxy.
 *
 */

// This interface is entirely internal to Servo, and should not be accessible to
// web pages.

[Pref="dom.testbinding.enabled", Exposed=(Window,Worker)]
interface TestBindingProxy : TestBinding {
  readonly attribute unsigned long length;

  getter DOMString getNamedItem(DOMString item_name);

  setter undefined setNamedItem(DOMString item_name, DOMString value);

  getter DOMString getItem(unsigned long index);

  setter undefined setItem(unsigned long index, DOMString value);

  deleter undefined removeItem(DOMString name);

  stringifier;
};
