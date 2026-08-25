/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
/*
 * The origin of this IDL file is
 * http://dev.w3.org/csswg/cssom/#the-cssstyledeclaration-interface
 *
 * Copyright © 2013 W3C® (MIT, ERCIM, Keio, Beihang), All Rights Reserved.
 */

[Exposed=Window]
interface CSSStyleDeclaration {
  [CEReactions, SetterThrows]
           attribute DOMString cssText;
  readonly attribute unsigned long length;
  getter DOMString item(unsigned long index);
  DOMString getPropertyValue(DOMString property);
  DOMString getPropertyPriority(DOMString property);
  [CEReactions, Throws]
  undefined setProperty(DOMString property, [LegacyNullToEmptyString] DOMString value,
                                       optional [LegacyNullToEmptyString] DOMString priority = "");
  [CEReactions, Throws]
  DOMString removeProperty(DOMString property);
  // readonly attribute CSSRule? parentRule;
  [CEReactions, SetterThrows]
           attribute DOMString cssFloat;
};

// Auto-generated in GlobalGen.py: accessors for each CSS property
