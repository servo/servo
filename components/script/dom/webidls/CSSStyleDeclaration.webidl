/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * The origin of this IDL file is
 * http://dev.w3.org/csswg/cssom/#the-cssstyledeclaration-interface
 *
 * Copyright © 2013 W3C® (MIT, ERCIM, Keio, Beihang), All Rights Reserved.
 */

interface CSSStyleDeclaration {
  //[SetterThrows]
  //         attribute DOMString cssText;
  readonly attribute unsigned long length;
  getter DOMString item(unsigned long index);
  DOMString getPropertyValue(DOMString property);
  //DOMString getPropertyPriority(DOMString property);
  [Throws]
  void setProperty(DOMString property, [TreatNullAs=EmptyString] DOMString value,
                                       [TreatNullAs=EmptyString] optional DOMString priority = "");
  [Throws]
  void setPropertyValue(DOMString property, [TreatNullAs=EmptyString] DOMString value);
  //[Throws]
  //void setPropertyPriority(DOMString property, [TreatNullAs=EmptyString] DOMString priority);
  DOMString removeProperty(DOMString property);
  //readonly attribute CSSRule? parentRule;
  [SetterThrows]
           attribute DOMString cssFloat;
};

