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
 * @fileoverview Definitions for IE's custom CSS properties, as defined here:
 * http://msdn.microsoft.com/en-us/library/aa768661(VS.85).aspx
 *
 * This page is also useful for the IDL definitions:
 * http://source.winehq.org/source/include/mshtml.idl
 *
 * @externs
 * @author nicksantos@google.com
 */

/** @type {Element} */
StyleSheet.prototype.owningElement;

/** @type {boolean} */
StyleSheet.prototype.readOnly;

/** @type {StyleSheetList} */
StyleSheet.prototype.imports;

/** @type {string} */
StyleSheet.prototype.id;

/**
 * @param {string} bstrURL
 * @param {number} lIndex
 * @return {number}
 */
StyleSheet.prototype.addImport;

/**
 * @param {string} bstrSelector
 * @param {string} bstrStyle
 * @param {number=} opt_iIndex
 * @return {number}
 * @see http://msdn.microsoft.com/en-us/library/aa358796%28v=vs.85%29.aspx
 */
StyleSheet.prototype.addRule;

/**
 * @param {number} lIndex
 */
StyleSheet.prototype.removeImport;

/**
 * @param {number} lIndex
 */
StyleSheet.prototype.removeRule;

/** @type {string} */
StyleSheet.prototype.cssText;

/** @type {CSSRuleList} */
StyleSheet.prototype.rules;

// StyleSheet methods

/**
 * @param {string} propName
 * @return {string}
 * @see http://msdn.microsoft.com/en-us/library/aa358797(VS.85).aspx
 */
StyleSheet.prototype.getExpression;

/**
 * @param {string} name
 * @param {string} expression
 * @return {undefined}
 * @see http://msdn.microsoft.com/en-us/library/ms531196(VS.85).aspx
 */
StyleSheet.prototype.setExpression;

/**
 * @param {string} expression
 * @return {undefined}
 * @see http://msdn.microsoft.com/en-us/library/aa358798(VS.85).aspx
 */
StyleSheet.prototype.removeExpression;

// IE-only CSS style names.

/** @type {string} */ CSSProperties.prototype.backgroundPositionX;

/** @type {string} */ CSSProperties.prototype.backgroundPositionY;

/**
 * @see http://msdn.microsoft.com/en-us/library/ie/ms531081(v=vs.85).aspx
 * NOTE: Left untyped to avoid conflict with caller.
 */
CSSProperties.prototype.behavior;

/**
 * @type {string}
 * @see http://msdn.microsoft.com/en-us/library/ms533883.aspx
 */
CSSProperties.prototype.imeMode;

/**
 * @type {string}
 * @see http://msdn.microsoft.com/en-us/library/ms534176(VS.85).aspx
 */
CSSProperties.prototype.msInterpolationMode;

/** @type {string} */ CSSProperties.prototype.overflowX;

/** @type {string} */ CSSProperties.prototype.overflowY;

/** @type {number} */ CSSProperties.prototype.pixelWidth;

/** @type {number} */ CSSProperties.prototype.pixelHeight;

/** @type {number} */ CSSProperties.prototype.pixelLeft;

/** @type {number} */ CSSProperties.prototype.pixelTop;

/** @type {string} */ CSSProperties.prototype.styleFloat;

/**
 * @type {string|number}
 * @see http://msdn.microsoft.com/en-us/library/ms535169(VS.85).aspx
 */
CSSProperties.prototype.zoom;

/**
 * @type {string}
 * @see http://msdn.microsoft.com/en-us/library/ms535153(VS.85).aspx
 */
CSSProperties.prototype.writingMode;

/**
 * IE-specific extensions.
 * @see http://blogs.msdn.com/b/ie/archive/2008/09/08/microsoft-css-vendor-extensions.aspx
 */

/** @type {string} */
CSSProperties.prototype.MsAccelerator;

/** @type {string} */
CSSProperties.prototype.MsBackgroundPositionX;

/** @type {string} */
CSSProperties.prototype.MsBackgroundPositionY;

/** @type {string} */
CSSProperties.prototype.MsBehavior;

/** @type {string} */
CSSProperties.prototype.MsBlockProgression;

/** @type {string} */
CSSProperties.prototype.MsFilter;

/** @type {string} */
CSSProperties.prototype.MsImeMode;

/** @type {string} */
CSSProperties.prototype.MsLayoutGrid;

/** @type {string} */
CSSProperties.prototype.MsLayoutGridChar;

/** @type {string} */
CSSProperties.prototype.MsLayoutGridLine;

/** @type {string} */
CSSProperties.prototype.MsLayoutGridMode;

/** @type {string} */
CSSProperties.prototype.MsLayoutGridType;

/** @type {string} */
CSSProperties.prototype.MsLineBreak;

/** @type {string} */
CSSProperties.prototype.MsLineGridMode;

/** @type {string} */
CSSProperties.prototype.MsInterpolationMode;

/** @type {string} */
CSSProperties.prototype.MsOverflowX;

/** @type {string} */
CSSProperties.prototype.MsOverflowY;

/** @type {string} */
CSSProperties.prototype.MsScrollbar3dlightColor;

/** @type {string} */
CSSProperties.prototype.MsScrollbarArrowColor;

/** @type {string} */
CSSProperties.prototype.MsScrollbarBaseColor;

/** @type {string} */
CSSProperties.prototype.MsScrollbarDarkshadowColor;

/** @type {string} */
CSSProperties.prototype.MsScrollbarFaceColor;

CSSProperties.prototype.MsScrollbarHighlightColor;

/** @type {string} */
CSSProperties.prototype.MsScrollbarShadowColor;

/** @type {string} */
CSSProperties.prototype.MsScrollbarTrackColor;

/** @type {string} */
CSSProperties.prototype.MsTextAlignLast;

/** @type {string} */
CSSProperties.prototype.MsTextAutospace;

/** @type {string} */
CSSProperties.prototype.MsTextJustify;

/** @type {string} */
CSSProperties.prototype.MsTextKashidaSpace;

/** @type {string} */
CSSProperties.prototype.MsTextOverflow;

/** @type {string} */
CSSProperties.prototype.MsTextUnderlinePosition;

/** @type {string} */
CSSProperties.prototype.MsWordBreak;

/** @type {string} */
CSSProperties.prototype.MsWordWrap;

/** @type {string} */
CSSProperties.prototype.MsWritingMode;

/** @type {string} */
CSSProperties.prototype.MsZoom;

// See: http://msdn.microsoft.com/en-us/library/windows/apps/Hh702466.aspx

/** @type {string} */
CSSProperties.prototype.msContentZooming;

/** @type {string} */
CSSProperties.prototype.msTouchAction;

/** @type {string} */
CSSProperties.prototype.msTransform;

/** @type {string} */
CSSProperties.prototype.msTransition;
