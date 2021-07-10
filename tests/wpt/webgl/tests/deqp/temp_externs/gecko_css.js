/*
 * Copyright 2008 The Closure Compiler Authors
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

/**
 * @fileoverview Definitions for Gecko's custom CSS properties. Copied from:
 * http://mxr.mozilla.org/mozilla2.0/source/dom/interfaces/css/nsIDOMCSS2Properties.idl
 *
 * @externs
 * @author nicksantos@google.com (Nick Santos)
 */


/** @type {string} */ CSSProperties.prototype.MozAppearance;
/** @type {string} */ CSSProperties.prototype.MozBackfaceVisibility;
/** @type {string} */ CSSProperties.prototype.MozBackgroundClip;
/** @type {string} */ CSSProperties.prototype.MozBackgroundInlinePolicy;
/** @type {string} */ CSSProperties.prototype.MozBackgroundOrigin;
/** @type {string} */ CSSProperties.prototype.MozBinding;
/** @type {string} */ CSSProperties.prototype.MozBorderBottomColors;
/** @type {string} */ CSSProperties.prototype.MozBorderEnd;
/** @type {string} */ CSSProperties.prototype.MozBorderEndColor;
/** @type {string} */ CSSProperties.prototype.MozBorderEndStyle;
/** @type {string} */ CSSProperties.prototype.MozBorderEndWidth;
/** @type {string} */ CSSProperties.prototype.MozBorderImage;
/** @type {string} */ CSSProperties.prototype.MozBorderLeftColors;
/** @type {string} */ CSSProperties.prototype.MozBorderRadius;
/** @type {string} */ CSSProperties.prototype.MozBorderRadiusTopleft;
/** @type {string} */ CSSProperties.prototype.MozBorderRadiusTopright;
/** @type {string} */ CSSProperties.prototype.MozBorderRadiusBottomleft;
/** @type {string} */ CSSProperties.prototype.MozBorderRadiusBottomright;
/** @type {string} */ CSSProperties.prototype.MozBorderRightColors;
/** @type {string} */ CSSProperties.prototype.MozBorderStart;
/** @type {string} */ CSSProperties.prototype.MozBorderStartColor;
/** @type {string} */ CSSProperties.prototype.MozBorderStartStyle;
/** @type {string} */ CSSProperties.prototype.MozBorderStartWidth;
/** @type {string} */ CSSProperties.prototype.MozBorderTopColors;
/** @type {string} */ CSSProperties.prototype.MozBoxAlign;
/** @type {string} */ CSSProperties.prototype.MozBoxDirection;
/** @type {string} */ CSSProperties.prototype.MozBoxFlex;
/** @type {string} */ CSSProperties.prototype.MozBoxOrdinalGroup;
/** @type {string} */ CSSProperties.prototype.MozBoxOrient;
/** @type {string} */ CSSProperties.prototype.MozBoxPack;
/** @type {string} */ CSSProperties.prototype.MozBoxSizing;
/** @type {string} */ CSSProperties.prototype.MozBoxShadow;
/** @type {string} */ CSSProperties.prototype.MozColumnCount;
/** @type {string} */ CSSProperties.prototype.MozColumnGap;
/** @type {string} */ CSSProperties.prototype.MozColumnRule;
/** @type {string} */ CSSProperties.prototype.MozColumnRuleColor;
/** @type {string} */ CSSProperties.prototype.MozColumnRuleStyle;
/** @type {string} */ CSSProperties.prototype.MozColumnRuleWidth;
/** @type {string} */ CSSProperties.prototype.MozColumnWidth;
/** @type {string} */ CSSProperties.prototype.MozFloatEdge;
/** @type {string} */ CSSProperties.prototype.MozFontFeatureSettings;
/** @type {string} */ CSSProperties.prototype.MozFontLanguageOverride;
/** @type {string} */ CSSProperties.prototype.MozForceBrokenImageIcon;
/** @type {string} */ CSSProperties.prototype.MozImageRegion;
/** @type {string} */ CSSProperties.prototype.MozMarginEnd;
/** @type {string} */ CSSProperties.prototype.MozMarginStart;
/** @type {number|string} */ CSSProperties.prototype.MozOpacity;
/** @type {string} */ CSSProperties.prototype.MozOutline;
/** @type {string} */ CSSProperties.prototype.MozOutlineColor;
/** @type {string} */ CSSProperties.prototype.MozOutlineOffset;
/** @type {string} */ CSSProperties.prototype.MozOutlineRadius;
/** @type {string} */ CSSProperties.prototype.MozOutlineRadiusBottomleft;
/** @type {string} */ CSSProperties.prototype.MozOutlineRadiusBottomright;
/** @type {string} */ CSSProperties.prototype.MozOutlineRadiusTopleft;
/** @type {string} */ CSSProperties.prototype.MozOutlineRadiusTopright;
/** @type {string} */ CSSProperties.prototype.MozOutlineStyle;
/** @type {string} */ CSSProperties.prototype.MozOutlineWidth;
/** @type {string} */ CSSProperties.prototype.MozPaddingEnd;
/** @type {string} */ CSSProperties.prototype.MozPaddingStart;
/** @type {string} */ CSSProperties.prototype.MozPerspective;
/** @type {string} */ CSSProperties.prototype.MozStackSizing;
/** @type {string} */ CSSProperties.prototype.MozTabSize;
/** @type {string} */ CSSProperties.prototype.MozTransform;
/** @type {string} */ CSSProperties.prototype.MozTransformOrigin;
/** @type {string} */ CSSProperties.prototype.MozTransition;
/** @type {string} */ CSSProperties.prototype.MozTransitionDelay;
/** @type {string} */ CSSProperties.prototype.MozTransitionDuration;
/** @type {string} */ CSSProperties.prototype.MozTransitionProperty;
/** @type {string} */ CSSProperties.prototype.MozTransitionTimingFunction;
/** @type {string} */ CSSProperties.prototype.MozUserFocus;
/** @type {string} */ CSSProperties.prototype.MozUserInput;
/** @type {string} */ CSSProperties.prototype.MozUserModify;
/** @type {string} */ CSSProperties.prototype.MozUserSelect;
/** @type {string} */ CSSProperties.prototype.MozWindowShadow;


// These are non-standard Gecko CSSOM properties on Window.prototype.screen.

/**
 * @type {number}
 * @see https://developer.mozilla.org/En/DOM/window.screen.availTop
 */
Screen.prototype.availTop;

/**
 * @type {number}
 * @see https://developer.mozilla.org/En/DOM/window.screen.availLeft
 */
Screen.prototype.availLeft;

/**
 * @type {number}
 * @see https://developer.mozilla.org/En/DOM/window.screen.left
 */
Screen.prototype.left;

/**
 * @type {number}
 * @see https://developer.mozilla.org/En/DOM/window.screen.top
 */
Screen.prototype.top;
