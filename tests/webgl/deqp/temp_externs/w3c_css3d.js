/*
 * Copyright 2010 The Closure Compiler Authors
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
 * @fileoverview Definitions for W3C's CSS 3D Transforms specification.
 *  The whole file has been fully type annotated. Created from
 *  https://dvcs.w3.org/hg/FXTF/raw-file/tip/matrix/index.html
 *
 * @externs
 */

/**
 * @constructor
 * @param {string=} opt_matrix
 * @see https://dvcs.w3.org/hg/FXTF/raw-file/tip/matrix/index.html#the-cssmatrix-interface
 */
function CSSMatrix(opt_matrix) {}

/**
 * @type {number}
 * @see https://dvcs.w3.org/hg/FXTF/raw-file/tip/matrix/index.html#three-dimensional-attributes
 */
CSSMatrix.prototype.m11;

/**
 * @type {number}
 * @see https://dvcs.w3.org/hg/FXTF/raw-file/tip/matrix/index.html#three-dimensional-attributes
 */
CSSMatrix.prototype.m12;

/**
 * @type {number}
 * @see https://dvcs.w3.org/hg/FXTF/raw-file/tip/matrix/index.html#three-dimensional-attributes
 */
CSSMatrix.prototype.m13;

/**
 * @type {number}
 * @see https://dvcs.w3.org/hg/FXTF/raw-file/tip/matrix/index.html#three-dimensional-attributes
 */
CSSMatrix.prototype.m14;

/**
 * @type {number}
 * @see https://dvcs.w3.org/hg/FXTF/raw-file/tip/matrix/index.html#three-dimensional-attributes
 */
CSSMatrix.prototype.m21;

/**
 * @type {number}
 * @see https://dvcs.w3.org/hg/FXTF/raw-file/tip/matrix/index.html#three-dimensional-attributes
 */
CSSMatrix.prototype.m22;

/**
 * @type {number}
 * @see https://dvcs.w3.org/hg/FXTF/raw-file/tip/matrix/index.html#three-dimensional-attributes
 */
CSSMatrix.prototype.m23;

/**
 * @type {number}
 * @see https://dvcs.w3.org/hg/FXTF/raw-file/tip/matrix/index.html#three-dimensional-attributes
 */
CSSMatrix.prototype.m24;

/**
 * @type {number}
 * @see https://dvcs.w3.org/hg/FXTF/raw-file/tip/matrix/index.html#three-dimensional-attributes
 */
CSSMatrix.prototype.m31;

/**
 * @type {number}
 * @see https://dvcs.w3.org/hg/FXTF/raw-file/tip/matrix/index.html#three-dimensional-attributes
 */
CSSMatrix.prototype.m32;

/**
 * @type {number}
 * @see https://dvcs.w3.org/hg/FXTF/raw-file/tip/matrix/index.html#three-dimensional-attributes
 */
CSSMatrix.prototype.m33;

/**
 * @type {number}
 * @see https://dvcs.w3.org/hg/FXTF/raw-file/tip/matrix/index.html#three-dimensional-attributes
 */
CSSMatrix.prototype.m34;

/**
 * @type {number}
 * @see https://dvcs.w3.org/hg/FXTF/raw-file/tip/matrix/index.html#three-dimensional-attributes
 */
CSSMatrix.prototype.m41;

/**
 * @type {number}
 * @see https://dvcs.w3.org/hg/FXTF/raw-file/tip/matrix/index.html#three-dimensional-attributes
 */
CSSMatrix.prototype.m42;

/**
 * @type {number}
 * @see https://dvcs.w3.org/hg/FXTF/raw-file/tip/matrix/index.html#three-dimensional-attributes
 */
CSSMatrix.prototype.m43;

/**
 * @type {number}
 * @see https://dvcs.w3.org/hg/FXTF/raw-file/tip/matrix/index.html#three-dimensional-attributes
 */
CSSMatrix.prototype.m44;

/**
 * @param {string} string
 * @return {void}
 */
CSSMatrix.prototype.setMatrixValue = function(string) {};

/**
 * @param {!CSSMatrix} secondMatrix
 * @return {!CSSMatrix}
 * @see https://dvcs.w3.org/hg/FXTF/raw-file/tip/matrix/index.html#widl-CSSMatrix-multiply-CSSMatrix-CSSMatrix-other
 */
CSSMatrix.prototype.multiply = function(secondMatrix) {};

/**
 * @return {CSSMatrix} Returns void if the matrix is non-invertable.
 * @see https://dvcs.w3.org/hg/FXTF/raw-file/tip/matrix/index.html#widl-CSSMatrix-inverse-CSSMatrix
 */
CSSMatrix.prototype.inverse = function() {};

/**
 * @param {number=} opt_x Defaults to 0.
 * @param {number=} opt_y Defaults to 0.
 * @param {number=} opt_z Defaults to 0.
 * @return {!CSSMatrix}
 * @see https://dvcs.w3.org/hg/FXTF/raw-file/tip/matrix/index.html#widl-CSSMatrix-translate-CSSMatrix-unrestricted-double-tx-unrestricted-double-ty-unrestricted-double-tz
 */
CSSMatrix.prototype.translate = function(opt_x, opt_y, opt_z) {};

/**
 * @param {number=} opt_scaleX Defaults to 1.
 * @param {number=} opt_scaleY Defaults to scaleX.
 * @param {number=} opt_scaleZ Defaults to 1.
 * @return {!CSSMatrix}
 * @see https://dvcs.w3.org/hg/FXTF/raw-file/tip/matrix/index.html#widl-CSSMatrix-scale-CSSMatrix-unrestricted-double-scale-unrestricted-double-originX-unrestricted-double-originY
 */
CSSMatrix.prototype.scale = function(opt_scaleX, opt_scaleY, opt_scaleZ) {};

/**
 * @param {number=} opt_rotX Defaults to 0.
 * @param {number=} opt_rotY Defaults to 0.
 * @param {number=} opt_rotZ Defaults to rotX if rotY is not defined, else 0.
 * @return {!CSSMatrix}
 * @see https://dvcs.w3.org/hg/FXTF/raw-file/tip/matrix/index.html#widl-CSSMatrix-rotate-CSSMatrix-unrestricted-double-angle-unrestricted-double-originX-unrestricted-double-originY
 */
CSSMatrix.prototype.rotate = function(opt_rotX, opt_rotY, opt_rotZ) {};

/**
 * @param {number=} opt_x Defaults to 0.
 * @param {number=} opt_y Defaults to 0.
 * @param {number=} opt_z Defaults to 0.
 * @param {number=} opt_angle Defaults to 0.
 * @return {!CSSMatrix}
 * @see https://dvcs.w3.org/hg/FXTF/raw-file/tip/matrix/index.html#widl-CSSMatrix-rotateAxisAngle-CSSMatrix-unrestricted-double-x-unrestricted-double-y-unrestricted-double-z-unrestricted-double-angle
 */
CSSMatrix.prototype.rotateAxisAngle =
    function(opt_x, opt_y, opt_z, opt_angle) {};

/**
 * @constructor
 * @param {string=} opt_matrix
 * @extends {CSSMatrix}
 * @see http://developer.apple.com/safari/library/documentation/AudioVideo/Reference/WebKitCSSMatrixClassReference/WebKitCSSMatrix/WebKitCSSMatrix.html#//apple_ref/javascript/instm/WebKitCSSMatrix/setMatrixValue
 */
function WebKitCSSMatrix(opt_matrix) {}

/**
 * @constructor
 * @param {string=} opt_matrix
 * @extends {CSSMatrix}
 * @see http://msdn.microsoft.com/en-us/library/windows/apps/hh453593.aspx
 */
function MSCSSMatrix(opt_matrix) {}
