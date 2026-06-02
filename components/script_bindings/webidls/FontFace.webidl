/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

typedef DOMString CSSOMString;

dictionary FontFaceDescriptors {
  CSSOMString style = "normal";
  CSSOMString weight = "normal";
  CSSOMString stretch = "normal";
  CSSOMString unicodeRange = "U+0-10FFFF";
  CSSOMString featureSettings = "normal";
  CSSOMString variationSettings = "normal";
  CSSOMString display = "auto";
  CSSOMString ascentOverride = "normal";
  CSSOMString descentOverride = "normal";
  CSSOMString lineGapOverride = "normal";
};

enum FontFaceLoadStatus { "unloaded", "loading", "loaded", "error" };

// https://drafts.csswg.org/css-font-loading/#fontface-interface
[Exposed=(Window /*, Worker */), Pref="dom_fontface_enabled"] // TODO: Add support for FontFace in Workers.
interface FontFace {
  constructor(CSSOMString family, (CSSOMString or BufferSource) source,
                optional FontFaceDescriptors descriptors = {});
  [SetterThrows]
  attribute CSSOMString family;
  [SetterThrows]
  attribute CSSOMString style;
  [SetterThrows]
  attribute CSSOMString weight;
  [SetterThrows]
  attribute CSSOMString stretch;
  [SetterThrows]
  attribute CSSOMString unicodeRange;
  [SetterThrows]
  attribute CSSOMString featureSettings;
  [SetterThrows]
  attribute CSSOMString variationSettings;
  [SetterThrows]
  attribute CSSOMString display;
  [SetterThrows]
  attribute CSSOMString ascentOverride;
  [SetterThrows]
  attribute CSSOMString descentOverride;
  [SetterThrows]
  attribute CSSOMString lineGapOverride;

  readonly attribute FontFaceLoadStatus status;

  Promise<FontFace> load();
  readonly attribute Promise<FontFace> loaded;
};
