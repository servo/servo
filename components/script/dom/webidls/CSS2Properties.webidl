/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * The origin of this IDL file is
 * http://dev.w3.org/csswg/cssom/
 *
 */

interface CSS2Properties : CSSStyleDeclaration {
  [TreatNullAs=EmptyString] attribute DOMString background;
  [TreatNullAs=EmptyString] attribute DOMString backgroundColor;
  [TreatNullAs=EmptyString] attribute DOMString backgroundPosition;
  [TreatNullAs=EmptyString] attribute DOMString backgroundRepeat;
  [TreatNullAs=EmptyString] attribute DOMString backgroundImage;
  [TreatNullAs=EmptyString] attribute DOMString backgroundAttachment;

  [TreatNullAs=EmptyString] attribute DOMString border;
  [TreatNullAs=EmptyString] attribute DOMString borderColor;
  [TreatNullAs=EmptyString] attribute DOMString borderStyle;
  [TreatNullAs=EmptyString] attribute DOMString borderWidth;
  [TreatNullAs=EmptyString] attribute DOMString borderBottom;
  [TreatNullAs=EmptyString] attribute DOMString borderBottomColor;
  [TreatNullAs=EmptyString] attribute DOMString borderBottomStyle;
  [TreatNullAs=EmptyString] attribute DOMString borderBottomWidth;
  [TreatNullAs=EmptyString] attribute DOMString borderLeft;
  [TreatNullAs=EmptyString] attribute DOMString borderLeftColor;
  [TreatNullAs=EmptyString] attribute DOMString borderLeftStyle;
  [TreatNullAs=EmptyString] attribute DOMString borderLeftWidth;
  [TreatNullAs=EmptyString] attribute DOMString borderRight;
  [TreatNullAs=EmptyString] attribute DOMString borderRightColor;
  [TreatNullAs=EmptyString] attribute DOMString borderRightStyle;
  [TreatNullAs=EmptyString] attribute DOMString borderRightWidth;
  [TreatNullAs=EmptyString] attribute DOMString borderTop;
  [TreatNullAs=EmptyString] attribute DOMString borderTopColor;
  [TreatNullAs=EmptyString] attribute DOMString borderTopStyle;
  [TreatNullAs=EmptyString] attribute DOMString borderTopWidth;

  [TreatNullAs=EmptyString] attribute DOMString content;

  [TreatNullAs=EmptyString] attribute DOMString color;

  [TreatNullAs=EmptyString] attribute DOMString display;

  [TreatNullAs=EmptyString] attribute DOMString visibility;

  //[TreatNullAs=EmptyString] attribute DOMString float; //XXXjdm need BinaryName annotation

  [TreatNullAs=EmptyString] attribute DOMString clear;

  [TreatNullAs=EmptyString] attribute DOMString direction;

  [TreatNullAs=EmptyString] attribute DOMString lineHeight;

  [TreatNullAs=EmptyString] attribute DOMString verticalAlign;

  [TreatNullAs=EmptyString] attribute DOMString overflow;

  [TreatNullAs=EmptyString] attribute DOMString tableLayout;

  [TreatNullAs=EmptyString] attribute DOMString whiteSpace;

  [TreatNullAs=EmptyString] attribute DOMString writingMode;

  [TreatNullAs=EmptyString] attribute DOMString textAlign;
  [TreatNullAs=EmptyString] attribute DOMString textDecoration;
  [TreatNullAs=EmptyString] attribute DOMString textOrientation;

  [TreatNullAs=EmptyString] attribute DOMString font;
  [TreatNullAs=EmptyString] attribute DOMString fontFamily;
  [TreatNullAs=EmptyString] attribute DOMString fontSize;
  [TreatNullAs=EmptyString] attribute DOMString fontStyle;
  [TreatNullAs=EmptyString] attribute DOMString fontVariant;
  [TreatNullAs=EmptyString] attribute DOMString fontWeight;

  [TreatNullAs=EmptyString] attribute DOMString margin;
  [TreatNullAs=EmptyString] attribute DOMString marginBottom;
  [TreatNullAs=EmptyString] attribute DOMString marginLeft;
  [TreatNullAs=EmptyString] attribute DOMString marginRight;
  [TreatNullAs=EmptyString] attribute DOMString marginTop;

  [TreatNullAs=EmptyString] attribute DOMString padding;
  [TreatNullAs=EmptyString] attribute DOMString paddingBottom;
  [TreatNullAs=EmptyString] attribute DOMString paddingLeft;
  [TreatNullAs=EmptyString] attribute DOMString paddingRight;
  [TreatNullAs=EmptyString] attribute DOMString paddingTop;

  [TreatNullAs=EmptyString] attribute DOMString position;

  [TreatNullAs=EmptyString] attribute DOMString top;
  [TreatNullAs=EmptyString] attribute DOMString right;
  [TreatNullAs=EmptyString] attribute DOMString left;
  [TreatNullAs=EmptyString] attribute DOMString bottom;

  [TreatNullAs=EmptyString] attribute DOMString height;
  [TreatNullAs=EmptyString] attribute DOMString minHeight;
  [TreatNullAs=EmptyString] attribute DOMString maxHeight;

  [TreatNullAs=EmptyString] attribute DOMString width;
  [TreatNullAs=EmptyString] attribute DOMString minWidth;
  [TreatNullAs=EmptyString] attribute DOMString maxWidth;
};
