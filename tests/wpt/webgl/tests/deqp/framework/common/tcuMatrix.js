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
goog.provide('framework.common.tcuMatrix');
goog.require('framework.delibs.debase.deMath');

goog.scope(function() {

    var tcuMatrix = framework.common.tcuMatrix;
    var deMath = framework.delibs.debase.deMath;

    var DE_ASSERT = function(x) {
        if (!x)
            throw new Error('Assert failed');
    };

    /**
     * @constructor
     * @param {number} rows
     * @param {number} cols
     * @param {*=} value
     * Initialize to identity.
     */
    tcuMatrix.Matrix = function(rows, cols, value) {
        value = value == undefined ? 1 : value;
        this.rows = rows;
        this.cols = cols;
        this.matrix = [];
        for (var i = 0; i < cols; i++)
            this.matrix[i] = [];
        for (var row = 0; row < rows; row++)
            for (var col = 0; col < cols; col++)
                this.set(row, col, (row == col) ? value : 0);
    };

    /**
     * @param {number} rows
     * @param {number} cols
     * @param {Array<number>} vector
     * @return {tcuMatrix.Matrix}
     */
    tcuMatrix.matrixFromVector = function(rows, cols, vector) {
        var matrix = new tcuMatrix.Matrix(rows, cols);
        for (var row = 0; row < vector.length; row++)
            for (var col = 0; col < vector.length; col++)
                matrix.matrix[col][row] = row == col ? vector[row] : 0;
        return matrix;
    };

    /**
     * @param {number} rows
     * @param {number} cols
     * @param {Array<number>} src
     * @return {tcuMatrix.Matrix}
     */
    tcuMatrix.matrixFromDataArray = function(rows, cols, src) {
        var matrix = new tcuMatrix.Matrix(rows, cols);
        for (var row = 0; row < rows; row++) {
            for (var col = 0; col < cols; col++) {
                matrix.matrix[col][row] = src[row * cols + col];
            }
        }
        return matrix;
    };

    /**
     * Fill the Matrix with data from array
     * @param {number} rows
     * @param {number} cols
     * @param {Array<number>} array
     * @return {tcuMatrix.Matrix}
     */
    tcuMatrix.matrixFromArray = function(rows, cols, array) {
        DE_ASSERT(array.length === rows * cols);
        var matrix = new tcuMatrix.Matrix(rows, cols);
        for (var row = 0; row < rows; row++)
            for (var col = 0; col < cols; col++)
                matrix.matrix[col][row] = array[row * cols + col];
        return matrix;
    };

    tcuMatrix.Matrix.prototype.set = function(x, y, value) {
        this.isRangeValid(x, y);
        this.matrix[y][x] = value;
    };

    tcuMatrix.Matrix.prototype.setRow = function(row, values) {
        if (!deMath.deInBounds32(row, 0, this.rows))
            throw new Error('Rows out of range');
        if (values.length > this.cols)
            throw new Error('Too many columns');
        for (var col = 0; col < values.length; col++)
            this.matrix[col][row] = values[col];
    };

    tcuMatrix.Matrix.prototype.setCol = function(col, values) {
        if (!deMath.deInBounds32(col, 0, this.cols))
            throw new Error('Columns out of range');
        if (values.length > this.rows)
            throw new Error('Too many rows');
        for (var row = 0; row < values.length; row++)
            this.matrix[col][row] = values[row];
    };

    tcuMatrix.Matrix.prototype.get = function(x, y) {
        this.isRangeValid(x, y);
        return this.matrix[y][x];
    };

    tcuMatrix.Matrix.prototype.getColumn = function(y) {
        return this.matrix[y];
    };

    tcuMatrix.Matrix.prototype.isRangeValid = function(x, y) {
        if (!deMath.deInBounds32(x, 0, this.rows))
            throw new Error('Rows out of range');
        if (!deMath.deInBounds32(y, 0, this.cols))
            throw new Error('Columns out of range');
    };

    /**
     * @return {Array<number>}
     */
    tcuMatrix.Matrix.prototype.getColumnMajorData = function() {
        /** @type {Array<number>} */ var a = [];
        for (var col = 0; col < this.cols; col++)
            for (var row = 0; row < this.rows; row++)
                a.push(this.get(row, col));
        return a;
    };

    /**
     * @param {tcuMatrix.Matrix} matrixA
     * @param {tcuMatrix.Matrix} matrixB
     * @return {tcuMatrix.Matrix}
     */
    tcuMatrix.add = function(matrixA, matrixB) {
        var res = new tcuMatrix.Matrix(matrixA.rows, matrixB.cols);
        for (var col = 0; col < matrixA.cols; col++)
            for (var row = 0; row < matrixA.rows; row++)
                res.set(row, col, matrixA.get(row, col) + matrixB.get(row, col));
        return res;
    };

    /**
     * @param {tcuMatrix.Matrix} matrixA
     * @param {tcuMatrix.Matrix} matrixB
     * @return {tcuMatrix.Matrix}
     */
    tcuMatrix.subtract = function(matrixA, matrixB) {
        var res = new tcuMatrix.Matrix(matrixA.rows, matrixB.cols);
        for (var col = 0; col < matrixA.cols; col++)
            for (var row = 0; row < matrixA.rows; row++)
                res.set(row, col, matrixA.get(row, col) - matrixB.get(row, col));
        return res;
    };

    /**
     * @param {tcuMatrix.Matrix} matrixA
     * @param {tcuMatrix.Matrix} matrixB
     * @return {tcuMatrix.Matrix}
     * Multiplication of two matrices.
     */
    tcuMatrix.multiply = function(matrixA, matrixB) {
        if (matrixA.cols != matrixB.rows)
            throw new Error('Wrong matrices sizes');
        var res = new tcuMatrix.Matrix(matrixA.rows, matrixB.cols);
        for (var row = 0; row < matrixA.rows; row++)
            for (var col = 0; col < matrixB.cols; col++) {
                var v = 0;
                for (var ndx = 0; ndx < matrixA.cols; ndx++)
                    v += matrixA.get(row, ndx) * matrixB.get(ndx, col);
                res.set(row, col, v);
            }
        return res;
    };

    /**
     * @param {tcuMatrix.Matrix} matrixA
     * @param {tcuMatrix.Matrix} matrixB
     * @return {tcuMatrix.Matrix}
     */
    tcuMatrix.divide = function(matrixA, matrixB) {
        var res = new tcuMatrix.Matrix(matrixA.rows, matrixA.cols);
        for (var col = 0; col < matrixA.cols; col++)
            for (var row = 0; row < matrixA.rows; row++)
                res.set(row, col, matrixA.get(row, col) / matrixB.get(row, col));
        return res;
    };

    /**
     * @param {tcuMatrix.Matrix} mtx
     * @param {Array<number>} vec
     * @return {Array<number>}
     */
    tcuMatrix.multiplyMatVec = function(mtx, vec) {
        /** @type {Array<number>} */ var res = [];
        /** @type {number} */ var value;
        for (var row = 0; row < mtx.rows; row++) {
            value = 0;
            for (var col = 0; col < mtx.cols; col++)
                value += mtx.get(row, col) * vec[col];
            res[row] = value;
        }

        return res;
    };

    /**
     * @param {Array<number>} vec
     * @param {tcuMatrix.Matrix} mtx
     * @return {Array<number>}
     */
    tcuMatrix.multiplyVecMat = function(vec, mtx) {
        /** @type {Array<number>} */ var res = [];
        /** @type {number} */ var value;
        for (var col = 0; col < mtx.cols; col++) {
            value = 0;
            for (var row = 0; row < mtx.rows; row++)
                value += mtx.get(row, col) * vec[row];
            res[col] = value;
        }

        return res;
    };

    tcuMatrix.Matrix.prototype.toString = function() {
        var str = 'mat' + this.cols;
        if (this.rows !== this.cols)
            str += 'x' + this.rows;
        str += '(';
        for (var col = 0; col < this.cols; col++) {
            str += '[';
            for (var row = 0; row < this.rows; row++) {
                str += this.matrix[col][row];
                if (row != this.rows - 1)
                    str += ', ';
            }
            str += ']';

            if (col != this.cols - 1)
                str += ', ';
        }
        str += ')';
        return str;
    };

    /**
     * @param {tcuMatrix.Matrix} mtx
     * @param {number} scalar
     * @return {tcuMatrix.Matrix}
     */
    tcuMatrix.subtractMatScal = function(mtx, scalar) {
        /** @type {tcuMatrix.Matrix} */ var res = new tcuMatrix.Matrix(mtx.rows, mtx.cols);
        for (var col = 0; col < mtx.cols; col++)
            for (var row = 0; row < mtx.rows; row++)
                res.set(row, col, mtx.get(row, col) - scalar);

        return res;
    };

    /**
     * @param {tcuMatrix.Matrix} mtx
     * @param {number} scalar
     * @return {tcuMatrix.Matrix}
     */
    tcuMatrix.addMatScal = function(mtx, scalar) {
        /** @type {tcuMatrix.Matrix} */ var res = new tcuMatrix.Matrix(mtx.rows, mtx.cols);
        for (var col = 0; col < mtx.cols; col++)
            for (var row = 0; row < mtx.rows; row++)
                res.set(row, col, mtx.get(row, col) + scalar);

        return res;
    };

    /**
     * @param {tcuMatrix.Matrix} mtx
     * @param {number} scalar
     * @return {tcuMatrix.Matrix}
     */
    tcuMatrix.multiplyMatScal = function(mtx, scalar) {
        /** @type {tcuMatrix.Matrix} */ var res = new tcuMatrix.Matrix(mtx.rows, mtx.cols);
        for (var col = 0; col < mtx.cols; col++)
            for (var row = 0; row < mtx.rows; row++)
                res.set(row, col, mtx.get(row, col) * scalar);

        return res;
    };

    /**
     * @param {tcuMatrix.Matrix} mtx
     * @param {number} scalar
     * @return {tcuMatrix.Matrix}
     */
    tcuMatrix.divideMatScal = function(mtx, scalar) {
        /** @type {tcuMatrix.Matrix} */ var res = new tcuMatrix.Matrix(mtx.rows, mtx.cols);
        for (var col = 0; col < mtx.cols; col++)
            for (var row = 0; row < mtx.rows; row++)
                res.set(row, col, mtx.get(row, col) / scalar);

        return res;
    };

    /**
     * @constructor
     * @extends {tcuMatrix.Matrix}
     */
    tcuMatrix.Mat2 = function() {
        tcuMatrix.Matrix.call(this, 2, 2);
    };

    tcuMatrix.Mat2.prototype = Object.create(tcuMatrix.Matrix.prototype);
    tcuMatrix.Mat2.prototype.constructor = tcuMatrix.Mat2;

    /**
     * @constructor
     * @extends {tcuMatrix.Matrix}
     */
    tcuMatrix.Mat3 = function() {
        tcuMatrix.Matrix.call(this, 3, 3);
    };

    tcuMatrix.Mat3.prototype = Object.create(tcuMatrix.Matrix.prototype);
    tcuMatrix.Mat3.prototype.constructor = tcuMatrix.Mat3;

    /**
     * @constructor
     * @extends {tcuMatrix.Matrix}
     */
    tcuMatrix.Mat4 = function() {
        tcuMatrix.Matrix.call(this, 4, 4);
    };

    tcuMatrix.Mat4.prototype = Object.create(tcuMatrix.Matrix.prototype);
    tcuMatrix.Mat4.prototype.constructor = tcuMatrix.Mat4;

});
