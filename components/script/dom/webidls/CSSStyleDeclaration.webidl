/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
/*
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
  DOMString getPropertyPriority(DOMString property);
  [Throws]
  void setProperty(DOMString property, [TreatNullAs=EmptyString] DOMString value,
                                       [TreatNullAs=EmptyString] optional DOMString priority = "");
  [Throws]
  void setPropertyValue(DOMString property, [TreatNullAs=EmptyString] DOMString value);

  [Throws]
  void setPropertyPriority(DOMString property, [TreatNullAs=EmptyString] DOMString priority);

  [Throws]
  DOMString removeProperty(DOMString property);
  //readonly attribute CSSRule? parentRule;
  [SetterThrows]
           attribute DOMString cssFloat;
};

partial interface CSSStyleDeclaration {
  [SetterThrows, TreatNullAs=EmptyString] attribute DOMString background;
  [SetterThrows, TreatNullAs=EmptyString] attribute DOMString backgroundColor;
  [SetterThrows, TreatNullAs=EmptyString] attribute DOMString backgroundPosition;
  [SetterThrows, TreatNullAs=EmptyString] attribute DOMString backgroundRepeat;
  [SetterThrows, TreatNullAs=EmptyString] attribute DOMString backgroundImage;
  [SetterThrows, TreatNullAs=EmptyString] attribute DOMString backgroundAttachment;
  [SetterThrows, TreatNullAs=EmptyString] attribute DOMString backgroundSize;
  [SetterThrows, TreatNullAs=EmptyString] attribute DOMString backgroundOrigin;
  [SetterThrows, TreatNullAs=EmptyString] attribute DOMString backgroundClip;

  [SetterThrows, TreatNullAs=EmptyString] attribute DOMString border;
  [SetterThrows, TreatNullAs=EmptyString] attribute DOMString borderColor;
  [SetterThrows, TreatNullAs=EmptyString] attribute DOMString borderRadius;
  [SetterThrows, TreatNullAs=EmptyString] attribute DOMString borderSpacing;
  [SetterThrows, TreatNullAs=EmptyString] attribute DOMString borderStyle;
  [SetterThrows, TreatNullAs=EmptyString] attribute DOMString borderWidth;
  [SetterThrows, TreatNullAs=EmptyString] attribute DOMString borderBottom;
  [SetterThrows, TreatNullAs=EmptyString] attribute DOMString borderBottomColor;
  [SetterThrows, TreatNullAs=EmptyString] attribute DOMString borderBottomLeftRadius;
  [SetterThrows, TreatNullAs=EmptyString] attribute DOMString borderBottomRightRadius;
  [SetterThrows, TreatNullAs=EmptyString] attribute DOMString borderBottomStyle;
  [SetterThrows, TreatNullAs=EmptyString] attribute DOMString borderBottomWidth;
  [SetterThrows, TreatNullAs=EmptyString] attribute DOMString borderLeft;
  [SetterThrows, TreatNullAs=EmptyString] attribute DOMString borderLeftColor;
  [SetterThrows, TreatNullAs=EmptyString] attribute DOMString borderLeftStyle;
  [SetterThrows, TreatNullAs=EmptyString] attribute DOMString borderLeftWidth;
  [SetterThrows, TreatNullAs=EmptyString] attribute DOMString borderRight;
  [SetterThrows, TreatNullAs=EmptyString] attribute DOMString borderRightColor;
  [SetterThrows, TreatNullAs=EmptyString] attribute DOMString borderRightStyle;
  [SetterThrows, TreatNullAs=EmptyString] attribute DOMString borderRightWidth;
  [SetterThrows, TreatNullAs=EmptyString] attribute DOMString borderTop;
  [SetterThrows, TreatNullAs=EmptyString] attribute DOMString borderTopColor;
  [SetterThrows, TreatNullAs=EmptyString] attribute DOMString borderTopLeftRadius;
  [SetterThrows, TreatNullAs=EmptyString] attribute DOMString borderTopRightRadius;
  [SetterThrows, TreatNullAs=EmptyString] attribute DOMString borderTopStyle;
  [SetterThrows, TreatNullAs=EmptyString] attribute DOMString borderTopWidth;

  [SetterThrows, TreatNullAs=EmptyString] attribute DOMString content;

  [SetterThrows, TreatNullAs=EmptyString] attribute DOMString color;

  [SetterThrows, TreatNullAs=EmptyString] attribute DOMString display;

  [SetterThrows, TreatNullAs=EmptyString] attribute DOMString opacity;

  [SetterThrows, TreatNullAs=EmptyString] attribute DOMString visibility;

  [SetterThrows, TreatNullAs=EmptyString] attribute DOMString cursor;

  [SetterThrows, TreatNullAs=EmptyString] attribute DOMString boxSizing;
  [SetterThrows, TreatNullAs=EmptyString] attribute DOMString boxShadow;
  [SetterThrows, TreatNullAs=EmptyString] attribute DOMString textShadow;

  [SetterThrows, TreatNullAs=EmptyString] attribute DOMString _float;

  [SetterThrows, TreatNullAs=EmptyString] attribute DOMString clear;

  [SetterThrows, TreatNullAs=EmptyString] attribute DOMString clip;

  [SetterThrows, TreatNullAs=EmptyString] attribute DOMString transform;
  [SetterThrows, TreatNullAs=EmptyString] attribute DOMString transformOrigin;
  [SetterThrows, TreatNullAs=EmptyString] attribute DOMString perspective;
  [SetterThrows, TreatNullAs=EmptyString] attribute DOMString perspectiveOrigin;
  [SetterThrows, TreatNullAs=EmptyString] attribute DOMString transformStyle;
  [SetterThrows, TreatNullAs=EmptyString] attribute DOMString backfaceVisibility;

  [SetterThrows, TreatNullAs=EmptyString] attribute DOMString direction;
  [SetterThrows, TreatNullAs=EmptyString] attribute DOMString unicodeBidi;

  [SetterThrows, TreatNullAs=EmptyString] attribute DOMString filter;

  [SetterThrows, TreatNullAs=EmptyString] attribute DOMString lineHeight;

  [SetterThrows, TreatNullAs=EmptyString] attribute DOMString mixBlendMode;

  [SetterThrows, TreatNullAs=EmptyString] attribute DOMString verticalAlign;

  [SetterThrows, TreatNullAs=EmptyString] attribute DOMString listStyle;
  [SetterThrows, TreatNullAs=EmptyString] attribute DOMString listStylePosition;
  [SetterThrows, TreatNullAs=EmptyString] attribute DOMString listStyleType;
  [SetterThrows, TreatNullAs=EmptyString] attribute DOMString listStyleImage;

  [SetterThrows, TreatNullAs=EmptyString] attribute DOMString quotes;

  [SetterThrows, TreatNullAs=EmptyString] attribute DOMString counterIncrement;
  [SetterThrows, TreatNullAs=EmptyString] attribute DOMString counterReset;

  [SetterThrows, TreatNullAs=EmptyString] attribute DOMString overflow;
  [SetterThrows, TreatNullAs=EmptyString] attribute DOMString overflowX;
  [SetterThrows, TreatNullAs=EmptyString] attribute DOMString overflowY;
  [SetterThrows, TreatNullAs=EmptyString] attribute DOMString overflowWrap;

  [SetterThrows, TreatNullAs=EmptyString] attribute DOMString tableLayout;
  [SetterThrows, TreatNullAs=EmptyString] attribute DOMString borderCollapse;
  [SetterThrows, TreatNullAs=EmptyString] attribute DOMString emptyCells;
  [SetterThrows, TreatNullAs=EmptyString] attribute DOMString captionSide;

  [SetterThrows, TreatNullAs=EmptyString] attribute DOMString whiteSpace;

  [SetterThrows, TreatNullAs=EmptyString] attribute DOMString writingMode;

  [SetterThrows, TreatNullAs=EmptyString] attribute DOMString letterSpacing;
  [SetterThrows, TreatNullAs=EmptyString] attribute DOMString wordBreak;
  [SetterThrows, TreatNullAs=EmptyString] attribute DOMString wordSpacing;
  [SetterThrows, TreatNullAs=EmptyString] attribute DOMString wordWrap;
  [SetterThrows, TreatNullAs=EmptyString] attribute DOMString textOverflow;

  [SetterThrows, TreatNullAs=EmptyString] attribute DOMString textAlign;
  [SetterThrows, TreatNullAs=EmptyString] attribute DOMString textDecoration;
  [SetterThrows, TreatNullAs=EmptyString] attribute DOMString textIndent;
  [SetterThrows, TreatNullAs=EmptyString] attribute DOMString textJustify;
  [SetterThrows, TreatNullAs=EmptyString] attribute DOMString textOrientation;
  [SetterThrows, TreatNullAs=EmptyString] attribute DOMString textRendering;
  [SetterThrows, TreatNullAs=EmptyString] attribute DOMString textTransform;

  [SetterThrows, TreatNullAs=EmptyString] attribute DOMString font;
  [SetterThrows, TreatNullAs=EmptyString] attribute DOMString fontFamily;
  [SetterThrows, TreatNullAs=EmptyString] attribute DOMString fontSize;
  [SetterThrows, TreatNullAs=EmptyString] attribute DOMString fontStretch;
  [SetterThrows, TreatNullAs=EmptyString] attribute DOMString fontStyle;
  [SetterThrows, TreatNullAs=EmptyString] attribute DOMString fontVariant;
  [SetterThrows, TreatNullAs=EmptyString] attribute DOMString fontWeight;

  [SetterThrows, TreatNullAs=EmptyString] attribute DOMString margin;
  [SetterThrows, TreatNullAs=EmptyString] attribute DOMString marginBottom;
  [SetterThrows, TreatNullAs=EmptyString] attribute DOMString marginLeft;
  [SetterThrows, TreatNullAs=EmptyString] attribute DOMString marginRight;
  [SetterThrows, TreatNullAs=EmptyString] attribute DOMString marginTop;

  [SetterThrows, TreatNullAs=EmptyString] attribute DOMString padding;
  [SetterThrows, TreatNullAs=EmptyString] attribute DOMString paddingBottom;
  [SetterThrows, TreatNullAs=EmptyString] attribute DOMString paddingLeft;
  [SetterThrows, TreatNullAs=EmptyString] attribute DOMString paddingRight;
  [SetterThrows, TreatNullAs=EmptyString] attribute DOMString paddingTop;

  [SetterThrows, TreatNullAs=EmptyString] attribute DOMString outline;
  [SetterThrows, TreatNullAs=EmptyString] attribute DOMString outlineColor;
  [SetterThrows, TreatNullAs=EmptyString] attribute DOMString outlineStyle;
  [SetterThrows, TreatNullAs=EmptyString] attribute DOMString outlineWidth;
  [SetterThrows, TreatNullAs=EmptyString] attribute DOMString outlineOffset;

  [SetterThrows, TreatNullAs=EmptyString] attribute DOMString position;

  [SetterThrows, TreatNullAs=EmptyString] attribute DOMString pointerEvents;

  [SetterThrows, TreatNullAs=EmptyString] attribute DOMString top;
  [SetterThrows, TreatNullAs=EmptyString] attribute DOMString right;
  [SetterThrows, TreatNullAs=EmptyString] attribute DOMString left;
  [SetterThrows, TreatNullAs=EmptyString] attribute DOMString bottom;

  [SetterThrows, TreatNullAs=EmptyString] attribute DOMString height;
  [SetterThrows, TreatNullAs=EmptyString] attribute DOMString minHeight;
  [SetterThrows, TreatNullAs=EmptyString] attribute DOMString maxHeight;

  [SetterThrows, TreatNullAs=EmptyString] attribute DOMString width;
  [SetterThrows, TreatNullAs=EmptyString] attribute DOMString minWidth;
  [SetterThrows, TreatNullAs=EmptyString] attribute DOMString maxWidth;

  [SetterThrows, TreatNullAs=EmptyString] attribute DOMString zIndex;

  [SetterThrows, TreatNullAs=EmptyString] attribute DOMString imageRendering;

  [SetterThrows, TreatNullAs=EmptyString] attribute DOMString columnCount;
  [SetterThrows, TreatNullAs=EmptyString] attribute DOMString columnWidth;
  [SetterThrows, TreatNullAs=EmptyString] attribute DOMString columns;
  [SetterThrows, TreatNullAs=EmptyString] attribute DOMString columnGap;

  [SetterThrows, TreatNullAs=EmptyString] attribute DOMString transition;
  [SetterThrows, TreatNullAs=EmptyString] attribute DOMString transitionDuration;
  [SetterThrows, TreatNullAs=EmptyString] attribute DOMString transitionTimingFunction;
  [SetterThrows, TreatNullAs=EmptyString] attribute DOMString transitionProperty;
  [SetterThrows, TreatNullAs=EmptyString] attribute DOMString transitionDelay;

  [SetterThrows, TreatNullAs=EmptyString] attribute DOMString flexDirection;
};
