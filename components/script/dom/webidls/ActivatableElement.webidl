/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// Interface for testing element activation
// This interface is entirely internal to Servo, and should not be accessible to
// web pages.
[Exposed=(Window,Worker), NoInterfaceObject]
interface ActivatableElement {
  [Throws, Pref="dom.testing.element.activation.enabled"]
  void enterFormalActivationState();

  [Throws, Pref="dom.testing.element.activation.enabled"]
  void exitFormalActivationState();
};
