/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://w3c.github.io/webvtt/#the-vttregion-interface

enum ScrollSetting { "" /* none */, "up"};

[Pref="dom.webvtt.enabled", Exposed=Window]
interface VTTRegion {
  [Throws] constructor();
  attribute DOMString id;
  [SetterThrows]
  attribute double width;
  [SetterThrows]
  attribute unsigned long lines;
  [SetterThrows]
  attribute double regionAnchorX;
  [SetterThrows]
  attribute double regionAnchorY;
  [SetterThrows]
  attribute double viewportAnchorX;
  [SetterThrows]
  attribute double viewportAnchorY;
  attribute ScrollSetting scroll;
};
