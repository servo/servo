/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://drafts.csswg.org/css-fonts/#cssfontfeaturevaluesrule

[Exposed=Window]
interface CSSFontFeatureValuesRule : CSSRule {
  // FIXME: This attribute should not be readonly. But assigning to it is unspecified behaviour.
  readonly attribute CSSOMString fontFamily;
  readonly attribute CSSFontFeatureValuesMap annotation;
  readonly attribute CSSFontFeatureValuesMap ornaments;
  readonly attribute CSSFontFeatureValuesMap stylistic;
  readonly attribute CSSFontFeatureValuesMap swash;
  readonly attribute CSSFontFeatureValuesMap characterVariant;
  readonly attribute CSSFontFeatureValuesMap styleset;

  // Note: This attribute exists in IDL but it does not make sense
  // and browsers don't support it: https://github.com/w3c/csswg-drafts/issues/13826
  // readonly attribute CSSFontFeatureValuesMap historicalForms;
};

[Exposed=Window]
interface CSSFontFeatureValuesMap {
  maplike<CSSOMString, sequence<unsigned long>>;
  // undefined set(CSSOMString featureValueName,
  //  (unsigned long or sequence<unsigned long>) values);
};
