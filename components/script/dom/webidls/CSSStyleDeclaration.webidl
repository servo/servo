/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */
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
                                       [TreatNullAs=EmptyString] optional DOMString priority = "");
  [CEReactions, Throws]
  void setPropertyValue(DOMString property, [TreatNullAs=EmptyString] DOMString value);

  [CEReactions, Throws]
  void setPropertyPriority(DOMString property, [TreatNullAs=EmptyString] DOMString priority);

  [CEReactions, Throws]
  DOMString removeProperty(DOMString property);
  // readonly attribute CSSRule? parentRule;
  [CEReactions, SetterThrows]
           attribute DOMString cssFloat;
};

partial interface CSSStyleDeclaration {
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString all;

  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString background;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString backgroundColor;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString background-color;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString backgroundPosition;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString background-position;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString backgroundPositionX;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString background-position-x;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString backgroundPositionY;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString background-position-y;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString backgroundRepeat;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString background-repeat;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString backgroundImage;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString background-image;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString backgroundAttachment;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString background-attachment;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString backgroundSize;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString background-size;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString backgroundOrigin;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString background-origin;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString backgroundClip;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString background-clip;

  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString border;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString borderColor;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString border-color;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString borderRadius;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString border-radius;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString borderSpacing;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString border-spacing;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString borderStyle;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString border-style;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString borderWidth;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString border-width;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString borderBottom;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString border-bottom;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString borderBottomColor;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString border-bottom-color;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString borderBottomLeftRadius;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString border-bottom-left-radius;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString borderBottomRightRadius;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString border-bottom-right-radius;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString borderBottomStyle;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString border-bottom-style;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString borderBottomWidth;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString border-bottom-width;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString borderLeft;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString border-left;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString borderLeftColor;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString border-left-color;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString borderLeftStyle;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString border-left-style;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString borderLeftWidth;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString border-left-width;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString borderRight;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString border-right;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString borderRightColor;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString border-right-color;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString borderRightStyle;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString border-right-style;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString borderRightWidth;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString border-right-width;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString borderTop;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString border-top;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString borderTopColor;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString border-top-color;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString borderTopLeftRadius;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString border-top-left-radius;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString borderTopRightRadius;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString border-top-right-radius;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString borderTopStyle;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString border-top-style;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString borderTopWidth;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString border-top-width;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString border-image-source;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString borderImageSource;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString border-image-slice;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString borderImageSlice;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString border-image-repeat;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString borderImageRepeat;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString border-image-outset;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString borderImageOutset;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString border-image-width;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString borderImageWidth;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString border-image;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString borderImage;

  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString border-block-start-color;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString borderBlockStartColor;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString border-block-start-width;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString borderBlockStartWidth;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString border-block-start-style;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString borderBlockStartStyle;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString border-block-end-color;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString borderBlockEndColor;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString border-block-end-width;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString borderBlockEndWidth;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString border-block-end-style;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString borderBlockEndStyle;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString border-inline-start-color;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString borderInlineStartColor;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString border-inline-start-width;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString borderInlineStartWidth;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString border-inline-start-style;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString borderInlineStartStyle;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString border-inline-end-color;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString borderInlineEndColor;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString border-inline-end-width;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString borderInlineEndWidth;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString border-inline-end-style;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString borderInlineEndStyle;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString border-block-start;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString borderBlockStart;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString border-block-end;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString borderBlockEnd;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString border-inline-start;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString borderInlineStart;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString border-inline-end;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString borderInlineEnd;

  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString content;

  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString color;

  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString display;

  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString opacity;

  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString visibility;

  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString cursor;

  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString boxSizing;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString box-sizing;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString boxShadow;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString box-shadow;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString textShadow;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString text-shadow;

  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString _float;

  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString clear;

  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString clip;

  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString transform;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString transformOrigin;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString transform-origin;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString perspective;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString perspectiveOrigin;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString perspective-origin;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString transformStyle;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString transform-style;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString backfaceVisibility;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString backface-visibility;

  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString direction;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString unicodeBidi;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString unicode-bidi;

  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString filter;

  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString lineHeight;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString line-height;

  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString mixBlendMode;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString mix-blend-mode;

  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString verticalAlign;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString vertical-align;

  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString listStyle;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString list-style;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString listStylePosition;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString list-style-position;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString listStyleType;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString list-style-type;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString listStyleImage;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString list-style-image;

  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString quotes;

  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString counterIncrement;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString counter-increment;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString counterReset;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString counter-reset;

  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString overflow;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString overflowX;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString overflow-x;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString overflowY;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString overflow-y;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString overflowWrap;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString overflow-wrap;

  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString tableLayout;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString table-layout;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString borderCollapse;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString border-collapse;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString emptyCells;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString empty-cells;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString captionSide;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString caption-side;

  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString whiteSpace;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString white-space;

  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString writingMode;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString writing-mode;

  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString letterSpacing;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString letter-spacing;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString wordBreak;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString word-break;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString wordSpacing;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString word-spacing;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString wordWrap;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString word-wrap;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString textOverflow;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString text-overflow;

  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString textAlign;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString text-align;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString textDecoration;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString text-decoration;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString textDecorationLine;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString text-decoration-line;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString textIndent;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString text-indent;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString textJustify;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString text-justify;
  // [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString textOrientation;
  // [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString text-orientation;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString textRendering;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString text-rendering;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString textTransform;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString text-transform;

  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString font;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString fontFamily;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString font-family;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString fontSize;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString font-size;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString fontStretch;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString font-stretch;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString fontStyle;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString font-style;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString fontVariant;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString font-variant;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString fontVariantCaps;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString font-variant-caps;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString fontWeight;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString font-weight;

  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString margin;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString marginBottom;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString margin-bottom;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString marginLeft;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString margin-left;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString marginRight;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString margin-right;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString marginTop;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString margin-top;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString margin-block-start;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString marginBlockStart;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString margin-block-end;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString marginBlockEnd;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString margin-inline-start;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString marginInlineStart;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString margin-inline-end;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString marginInlineEnd;

  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString padding;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString paddingBottom;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString padding-bottom;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString paddingLeft;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString padding-left;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString paddingRight;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString padding-right;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString paddingTop;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString padding-top;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString padding-block-start;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString paddingBlockStart;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString padding-block-end;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString paddingBlockEnd;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString padding-inline-start;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString paddingInlineStart;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString padding-inline-end;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString paddingInlineEnd;

  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString outline;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString outlineColor;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString outline-color;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString outlineStyle;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString outline-style;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString outlineWidth;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString outline-width;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString outlineOffset;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString outline-offset;

  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString position;

  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString pointerEvents;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString pointer-events;

  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString top;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString right;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString left;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString bottom;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString offset-block-start;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString offsetBlockStart;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString offset-block-end;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString offsetBlockEnd;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString offset-inline-start;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString offsetInlineStart;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString offset-inline-end;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString offsetInlineEnd;

  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString height;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString minHeight;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString min-height;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString maxHeight;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString max-height;

  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString width;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString minWidth;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString min-width;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString maxWidth;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString max-width;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString block-size;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString blockSize;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString inline-size;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString inlineSize;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString max-block-size;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString maxBlockSize;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString max-inline-size;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString maxInlineSize;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString min-block-size;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString minBlockSize;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString min-inline-size;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString minInlineSize;

  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString zIndex;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString z-index;

  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString imageRendering;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString image-rendering;

  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString columnCount;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString column-count;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString columnWidth;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString column-width;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString columns;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString columnGap;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString column-gap;

  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString transition;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString transitionDuration;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString transition-duration;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString transitionTimingFunction;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString transition-timing-function;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString transitionProperty;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString transition-property;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString transitionDelay;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString transition-delay;

  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString flex;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString flexFlow;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString flex-flow;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString flexDirection;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString flex-direction;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString flexWrap;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString flex-wrap;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString justifyContent;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString justify-content;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString alignItems;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString align-items;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString alignContent;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString align-content;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString order;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString flexBasis;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString flex-basis;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString flexGrow;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString flex-grow;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString flexShrink;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString flex-shrink;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString alignSelf;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString align-self;

  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString animation;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString animation-name;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString animationName;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString animation-duration;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString animationDuration;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString animation-timing-function;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString animationTimingFunction;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString animation-iteration-count;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString animationIterationCount;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString animation-direction;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString animationDirection;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString animation-play-state;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString animationPlayState;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString animation-fill-mode;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString animationFillMode;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString animation-delay;
  [CEReactions, SetterThrows, TreatNullAs=EmptyString] attribute DOMString animationDelay;
};
