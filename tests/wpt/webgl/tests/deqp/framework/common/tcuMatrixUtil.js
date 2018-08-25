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
goog.provide('framework.common.tcuMatrixUtil');
goog.require('framework.common.tcuMatrix');

goog.scope(function() {

    var tcuMatrixUtil = framework.common.tcuMatrixUtil;
    var tcuMatrix = framework.common.tcuMatrix;

    /**
     * @param {Array<number>} translation
     * @return {tcuMatrix.Matrix}
     */
    tcuMatrixUtil.translationMatrix = function(translation) {
        var len = translation.length;
        var res = new tcuMatrix.Matrix(len + 1, len + 1);
        for (var row = 0; row < len; row++)
            res.set(row, len, translation[row]);
        return res;
    };

    /**
     * Flatten an array of arrays or matrices
     * @param {(Array<Array<number>> | Array<tcuMatrix.Matrix>)} a
     * @return {Array<number>}
     */
    tcuMatrixUtil.flatten = function(a) {
        if (a[0] instanceof Array) {
            var merged = [];
            return merged.concat.apply(merged, a);
        }

        if (a[0] instanceof tcuMatrix.Matrix) {
            /** @type {tcuMatrix.Matrix} */ var m = a[0];
            var rows = m.rows;
            var cols = m.cols;
            var size = a.length;
            var result = [];
            for (var col = 0; col < cols; col++)
                for (var i = 0; i < size; i++)
                    result.push(a[i].getColumn(col));
            return [].concat.apply([], result);
        }

        if (typeof(a[0]) === 'number')
            return a;

        throw new Error('Invalid input');
    };

});
