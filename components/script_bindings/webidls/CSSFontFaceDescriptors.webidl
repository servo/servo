/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://drafts.csswg.org/css-fonts/#om-fontface

// FIXME: These fields should not be readonly
[Exposed=Window]
interface CSSFontFaceDescriptors : CSSStyleDeclaration {
  readonly attribute /* [LegacyNullToEmptyString] */ CSSOMString src;
  readonly attribute /* [LegacyNullToEmptyString] */ CSSOMString fontFamily;
  readonly attribute /* [LegacyNullToEmptyString] */ CSSOMString font-family;
  readonly attribute /* [LegacyNullToEmptyString] */ CSSOMString fontStyle;
  readonly attribute /* [LegacyNullToEmptyString] */ CSSOMString font-style;
  readonly attribute /* [LegacyNullToEmptyString] */ CSSOMString fontWeight;
  readonly attribute /* [LegacyNullToEmptyString] */ CSSOMString font-weight;
  readonly attribute /* [LegacyNullToEmptyString] */ CSSOMString fontStretch;
  readonly attribute /* [LegacyNullToEmptyString] */ CSSOMString font-stretch;
  readonly attribute /* [LegacyNullToEmptyString] */ CSSOMString fontWidth;
  readonly attribute /* [LegacyNullToEmptyString] */ CSSOMString font-width;
  readonly attribute /* [LegacyNullToEmptyString] */ CSSOMString unicodeRange;
  readonly attribute /* [LegacyNullToEmptyString] */ CSSOMString unicode-range;
  readonly attribute /* [LegacyNullToEmptyString] */ CSSOMString fontFeatureSettings;
  readonly attribute /* [LegacyNullToEmptyString] */ CSSOMString font-feature-settings;
  readonly attribute /* [LegacyNullToEmptyString] */ CSSOMString fontVariationSettings;
  readonly attribute /* [LegacyNullToEmptyString] */ CSSOMString font-variation-settings;
  // attribute [LegacyNullToEmptyString] CSSOMString fontNamedInstance;
  // attribute [LegacyNullToEmptyString] CSSOMString font-named-instance;
  readonly attribute /* [LegacyNullToEmptyString] */ CSSOMString fontDisplay;
  readonly attribute /* [LegacyNullToEmptyString] */ CSSOMString font-display;
  readonly attribute /* [LegacyNullToEmptyString] */ CSSOMString fontLanguageOverride;
  readonly attribute /* [LegacyNullToEmptyString] */ CSSOMString font-language-override;
  readonly attribute /* [LegacyNullToEmptyString] */ CSSOMString ascentOverride;
  readonly attribute /* [LegacyNullToEmptyString] */ CSSOMString ascent-override;
  readonly attribute /* [LegacyNullToEmptyString] */ CSSOMString descentOverride;
  readonly attribute /* [LegacyNullToEmptyString] */ CSSOMString descent-override;
  readonly attribute /* [LegacyNullToEmptyString] */ CSSOMString lineGapOverride;
  readonly attribute /* [LegacyNullToEmptyString] */ CSSOMString line-gap-override;
};
