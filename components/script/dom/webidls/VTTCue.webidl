/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://w3c.github.io/webvtt/#the-vttcue-interface

enum AutoKeyword { "auto"};
typedef (double or AutoKeyword) LineAndPositionSetting;
enum DirectionSetting { "" /* horizontal */, "rl", "lr" };
enum LineAlignSetting { "start", "center", "end" };
enum PositionAlignSetting { "line-left", "center", "line-right", "auto" };
enum AlignSetting { "start", "center", "end", "left", "right" };

[Pref="dom.webvtt.enabled", Exposed=Window]
interface VTTCue : TextTrackCue {
  constructor(double startTime, double endTime, DOMString text);
  attribute VTTRegion? region;
  attribute DirectionSetting vertical;
  attribute boolean snapToLines;
  attribute LineAndPositionSetting line;
  attribute LineAlignSetting lineAlign;
  [SetterThrows]
  attribute LineAndPositionSetting position;
  attribute PositionAlignSetting positionAlign;
  [SetterThrows]
  attribute double size;
  attribute AlignSetting align;
  attribute DOMString text;
  DocumentFragment getCueAsHTML();
};
