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
  void setProperty(DOMString property, [TreatNullAs=EmptyString] DOMString value,
                                       optional [TreatNullAs=EmptyString] DOMString priority = "");
  [CEReactions, Throws]
  DOMString removeProperty(DOMString property);
  // readonly attribute CSSRule? parentRule;
  [CEReactions, SetterThrows]
           attribute DOMString cssFloat;
};

partial interface CSSStyleDeclaration {
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString all;

  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString background;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString backgroundColor;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString background-color;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString backgroundPosition;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString background-position;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString backgroundPositionX;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString background-position-x;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString backgroundPositionY;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString background-position-y;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString backgroundRepeat;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString background-repeat;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString backgroundImage;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString background-image;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString backgroundAttachment;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString background-attachment;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString backgroundSize;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString background-size;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString backgroundOrigin;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString background-origin;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString backgroundClip;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString background-clip;

  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString border;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString borderColor;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString border-color;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString borderRadius;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString border-radius;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString borderSpacing;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString border-spacing;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString borderStyle;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString border-style;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString borderWidth;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString border-width;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString borderBottom;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString border-bottom;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString borderBottomColor;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString border-bottom-color;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString borderBottomLeftRadius;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString border-bottom-left-radius;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString borderBottomRightRadius;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString border-bottom-right-radius;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString borderBottomStyle;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString border-bottom-style;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString borderBottomWidth;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString border-bottom-width;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString borderLeft;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString border-left;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString borderLeftColor;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString border-left-color;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString borderLeftStyle;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString border-left-style;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString borderLeftWidth;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString border-left-width;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString borderRight;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString border-right;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString borderRightColor;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString border-right-color;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString borderRightStyle;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString border-right-style;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString borderRightWidth;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString border-right-width;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString borderTop;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString border-top;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString borderTopColor;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString border-top-color;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString borderTopLeftRadius;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString border-top-left-radius;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString borderTopRightRadius;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString border-top-right-radius;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString borderTopStyle;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString border-top-style;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString borderTopWidth;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString border-top-width;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString border-image-source;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString borderImageSource;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString border-image-slice;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString borderImageSlice;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString border-image-repeat;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString borderImageRepeat;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString border-image-outset;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString borderImageOutset;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString border-image-width;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString borderImageWidth;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString border-image;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString borderImage;

  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString border-block-start-color;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString borderBlockStartColor;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString border-block-start-width;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString borderBlockStartWidth;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString border-block-start-style;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString borderBlockStartStyle;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString border-block-end-color;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString borderBlockEndColor;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString border-block-end-width;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString borderBlockEndWidth;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString border-block-end-style;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString borderBlockEndStyle;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString border-block-color;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString borderBlockColor;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString border-block-style;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString borderBlockStyle;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString border-block-width;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString borderBlockWidth;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString border-block-end;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString borderBlockEnd;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString border-block-start;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString borderBlockStart;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString border-block;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString borderBlock;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString border-inline-start-color;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString borderInlineStartColor;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString border-inline-start-width;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString borderInlineStartWidth;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString border-inline-start-style;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString borderInlineStartStyle;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString border-inline-end-color;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString borderInlineEndColor;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString border-inline-end-width;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString borderInlineEndWidth;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString border-inline-end-style;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString borderInlineEndStyle;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString border-inline-color;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString borderInlineColor;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString border-inline-style;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString borderInlineStyle;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString border-inline-width;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString borderInlineWidth;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString border-inline-start;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString borderInlineStart;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString border-inline-end;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString borderInlineEnd;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString border-inline;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString borderInline;

  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString content;

  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString color;

  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString display;

  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString opacity;

  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString visibility;

  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString cursor;

  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString boxSizing;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString box-sizing;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString boxShadow;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString box-shadow;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString textShadow;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString text-shadow;

  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString _float;

  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString clear;

  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString clip;

  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString transform;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString transformOrigin;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString transform-origin;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString perspective;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString perspectiveOrigin;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString perspective-origin;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString transformStyle;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString transform-style;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString backfaceVisibility;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString backface-visibility;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString rotate;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString scale;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString translate;

  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString direction;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString unicodeBidi;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString unicode-bidi;

  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString filter;

  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString inset;

  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString lineHeight;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString line-height;

  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString mixBlendMode;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString mix-blend-mode;

  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString verticalAlign;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString vertical-align;

  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString listStyle;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString list-style;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString listStylePosition;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString list-style-position;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString listStyleType;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString list-style-type;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString listStyleImage;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString list-style-image;

  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString quotes;

  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString counterIncrement;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString counter-increment;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString counterReset;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString counter-reset;

  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString overflow;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString overflowX;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString overflow-x;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString overflowY;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString overflow-y;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString overflowWrap;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString overflow-wrap;

  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString tableLayout;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString table-layout;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString borderCollapse;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString border-collapse;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString emptyCells;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString empty-cells;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString captionSide;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString caption-side;

  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString whiteSpace;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString white-space;

  [Pref="layout.writing-mode.enabled", CEReactions, SetterThrows]
  attribute [TreatNullAs=EmptyString] DOMString writingMode;
  [Pref="layout.writing-mode.enabled", CEReactions, SetterThrows]
  attribute [TreatNullAs=EmptyString] DOMString writing-mode;

  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString letterSpacing;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString letter-spacing;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString wordBreak;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString word-break;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString wordSpacing;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString word-spacing;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString wordWrap;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString word-wrap;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString textOverflow;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString text-overflow;

  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString textAlign;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString text-align;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString textDecoration;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString text-decoration;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString textDecorationLine;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString text-decoration-line;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString textIndent;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString text-indent;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString textJustify;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString text-justify;
  // [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString textOrientation;
  // [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString text-orientation;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString textRendering;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString text-rendering;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString textTransform;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString text-transform;

  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString font;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString fontFamily;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString font-family;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString fontSize;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString font-size;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString fontStretch;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString font-stretch;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString fontStyle;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString font-style;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString fontVariant;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString font-variant;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString fontVariantCaps;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString font-variant-caps;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString fontWeight;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString font-weight;

  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString margin;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString marginBottom;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString margin-bottom;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString marginLeft;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString margin-left;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString marginRight;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString margin-right;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString marginTop;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString margin-top;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString margin-block-start;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString marginBlockStart;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString margin-block-end;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString marginBlockEnd;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString margin-block;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString marginBlock;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString margin-inline-start;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString marginInlineStart;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString margin-inline-end;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString marginInlineEnd;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString margin-inline;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString marginInline;

  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString padding;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString paddingBottom;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString padding-bottom;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString paddingLeft;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString padding-left;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString paddingRight;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString padding-right;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString paddingTop;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString padding-top;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString padding-block-start;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString paddingBlockStart;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString padding-block-end;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString paddingBlockEnd;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString padding-block;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString paddingBlock;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString padding-inline-start;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString paddingInlineStart;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString padding-inline-end;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString paddingInlineEnd;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString padding-inline;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString paddingInline;

  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString outline;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString outlineColor;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString outline-color;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString outlineStyle;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString outline-style;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString outlineWidth;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString outline-width;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString outlineOffset;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString outline-offset;

  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString position;

  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString pointerEvents;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString pointer-events;

  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString top;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString right;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString left;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString bottom;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString offset-block-start;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString offsetBlockStart;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString offset-block-end;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString offsetBlockEnd;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString offset-inline-start;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString offsetInlineStart;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString offset-inline-end;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString offsetInlineEnd;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString inset-block-start;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString insetBlockStart;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString inset-block-end;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString insetBlockEnd;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString inset-block;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString insetBlock;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString inset-inline-start;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString insetInlineStart;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString inset-inline-end;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString insetInlineEnd;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString inset-inline;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString insetInline;

  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString height;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString minHeight;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString min-height;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString maxHeight;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString max-height;

  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString width;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString minWidth;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString min-width;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString maxWidth;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString max-width;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString block-size;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString blockSize;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString inline-size;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString inlineSize;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString max-block-size;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString maxBlockSize;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString max-inline-size;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString maxInlineSize;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString min-block-size;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString minBlockSize;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString min-inline-size;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString minInlineSize;

  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString zIndex;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString z-index;

  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString imageRendering;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString image-rendering;

  [Pref="layout.columns.enabled", CEReactions, SetterThrows]
  attribute [TreatNullAs=EmptyString] DOMString columnCount;
  [Pref="layout.columns.enabled", CEReactions, SetterThrows]
  attribute [TreatNullAs=EmptyString] DOMString column-count;
  [Pref="layout.columns.enabled", CEReactions, SetterThrows]
  attribute [TreatNullAs=EmptyString] DOMString columnWidth;
  [Pref="layout.columns.enabled", CEReactions, SetterThrows]
  attribute [TreatNullAs=EmptyString] DOMString column-width;
  [Pref="layout.columns.enabled", CEReactions, SetterThrows]
  attribute [TreatNullAs=EmptyString] DOMString columns;
  [Pref="layout.columns.enabled", CEReactions, SetterThrows]
  attribute [TreatNullAs=EmptyString] DOMString columnGap;
  [Pref="layout.columns.enabled", CEReactions, SetterThrows]
  attribute [TreatNullAs=EmptyString] DOMString column-gap;

  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString transition;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString transitionDuration;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString transition-duration;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString transitionTimingFunction;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString transition-timing-function;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString transitionProperty;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString transition-property;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString transitionDelay;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString transition-delay;

  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString flex;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString flexFlow;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString flex-flow;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString flexDirection;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString flex-direction;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString flexWrap;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString flex-wrap;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString justifyContent;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString justify-content;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString alignItems;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString align-items;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString alignContent;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString align-content;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString order;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString flexBasis;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString flex-basis;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString flexGrow;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString flex-grow;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString flexShrink;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString flex-shrink;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString alignSelf;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString align-self;

  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString animation;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString animation-name;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString animationName;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString animation-duration;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString animationDuration;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString animation-timing-function;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString animationTimingFunction;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString animation-iteration-count;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString animationIterationCount;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString animation-direction;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString animationDirection;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString animation-play-state;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString animationPlayState;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString animation-fill-mode;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString animationFillMode;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString animation-delay;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString animationDelay;

  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString border-end-end-radius;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString borderEndEndRadius;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString border-start-end-radius;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString borderStartEndRadius;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString border-start-start-radius;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString borderStartStartRadius;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString border-end-start-radius;
  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString borderEndStartRadius;

};
