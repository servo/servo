/*-------------------------------------------------------------------------
 * drawElements Quality Program OpenGL ES Utilities
 * ------------------------------------------------
 *
 * Copyright 2014 The Android Open Source Project
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *      http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 *
 */

'use strict';
goog.provide('framework.referencerenderer.rrGenericVector');

goog.scope(function() {

var rrGenericVector = framework.referencerenderer.rrGenericVector;

    var DE_ASSERT = function(x) {
        if (!x)
            throw new Error('Assert failed');
    };

    /**
     * rrGenericVector.GenericVecType
     * @enum
     */
    rrGenericVector.GenericVecType = {
        FLOAT: 0,
        UINT32: 1,
        INT32: 2
    };

    /**
     * @constructor
     * @param {number=} a
     * @param {number=} b
     * @param {number=} c
     * @param {number=} d
     */
    rrGenericVector.GenericVec4 = function(a, b, c, d) {
        this.data = [a || 0, b || 0, c || 0, d || 0];
    };

});
