/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// Interface for testing element activation
// This interface is entirely internal to Servo, and should not be accessible to
// web pages.
[NoInterfaceObject]
interface ActivatableElement {
  [Pref="dom.testing.element.activation.enabled"]
  void enter_formal_activation_state();

  [Pref="dom.testing.element.activation.enabled"]
  void exit_formal_activation_state();
};
