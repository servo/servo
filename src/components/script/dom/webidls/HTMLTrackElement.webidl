/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * The origin of this IDL file is
 * http://www.whatwg.org/specs/web-apps/current-work/#the-track-element
 */

// import from http://mxr.mozilla.org/mozilla-central/source/dom/webidl/

/*
[Pref="media.webvtt.enabled"]
*/
interface HTMLTrackElement : HTMLElement {
  [SetterThrows, Pure]
  attribute DOMString kind;
  [SetterThrows, Pure]
  attribute DOMString src;
  [SetterThrows, Pure]
  attribute DOMString srclang;
  [SetterThrows, Pure]
  attribute DOMString label;
  [SetterThrows, Pure]
  attribute boolean default;

  const unsigned short NONE = 0;
  const unsigned short LOADING = 1;
  const unsigned short LOADED = 2;
  const unsigned short ERROR = 3;
  readonly attribute unsigned short readyState;
/*
  TODO:
  readonly attribute TextTrack track;
*/
};
