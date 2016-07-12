/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// This interface is entirely internal to Servo, and should not be accessible to
// web pages.

// FIXME: find a better way to hide this from content (#3688)
[NoInterfaceObject, Exposed=(Window,Worker)]
interface ServoHTMLParser {
};
