/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://drafts.csswg.org/cssom-view/#the-visualviewport-interface

[Exposed=Window]
interface VisualViewport : EventTarget {
  readonly attribute double offsetLeft;
  readonly attribute double offsetTop;

  readonly attribute double pageLeft;
  readonly attribute double pageTop;

  readonly attribute double width;
  readonly attribute double height;

  readonly attribute double scale;

  attribute EventHandler onresize;
  attribute EventHandler onscroll;
  attribute EventHandler onscrollend;
};
