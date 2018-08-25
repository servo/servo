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
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 *
 */
'use strict';
goog.provide('functional.gles3.es3fShaderMatrixTest');
goog.require('framework.opengl.gluShaderUtil');
goog.require('modules.shared.glsShaderRenderCase');
goog.require('framework.common.tcuMatrix');
goog.require('framework.delibs.debase.deMath');
goog.require('framework.common.tcuTestCase');

goog.scope(function() {

    var es3fShaderMatrixTest= functional.gles3.es3fShaderMatrixTest;
    var gluShaderUtil = framework.opengl.gluShaderUtil;
    var glsShaderRenderCase = modules.shared.glsShaderRenderCase;
    var tcuMatrix = framework.common.tcuMatrix;
    var deMath = framework.delibs.debase.deMath;
    var tcuTestCase = framework.common.tcuTestCase;


    /** @const {Array<number>}*/ var s_constInFloat = [0.5, -0.2];
    /** @const {Array<Array<number>>}*/ var s_constInVec2 = [[1.2, 0.5], [0.5, 1.0]];
    /** @const {Array<Array<number>>}*/ var s_constInVec3 = [[1.1, 0.1, 0.5], [-0.2, 0.5, 0.8]];
    /** @const {Array<Array<number>>}*/ var s_constInVec4 = [[1.4, 0.2, -0.5, 0.7], [0.2, -1.0, 0.5, 0.8]];

    /** @typedef {function(glsShaderRenderCase.ShaderEvalContext)} */ es3fShaderMatrixTest.MatrixShaderEvalFunc;

    /** @const {Array<Array<number>>}*/ var s_constInMat2x2 = [
        [-0.1, 1.0, -0.2, 0.0],
        [0.8, 0.1, 0.5, -0.9]
    ];
    /** @const {Array<Array<number>>}*/ var s_constInMat3x2 = [
        [0.8, -0.3, 0.3, 1.0,  1.2, -1.2],
	    [1.2, -1.0, 0.5, -0.8, 1.1, 0.3]
    ];

    /** @const {Array<Array<number>>}*/ var s_constInMat4x2 = [
		[-0.2,  0.5, 0.0, -1.0, 1.2, -0.5, 0.3, -0.9],
        [1.0,  0.1, -1.1,  0.6, 0.8, -1.2, -1.1,  0.7]
    ];

    /** @const {Array<Array<number>>}*/ var s_constInMat2x3 = [
        [-0.6, -0.1, -0.7, -1.2, -0.2, 0.0],
        [1.1, 0.6, 0.8, 1.0, 0.7, 0.1]
    ];

    /** @const {Array<Array<number>>}*/ var s_constInMat3x3 = [
        [-0.2,  1.1, 1.2, -1.0, 1.2, 0.5, 0.7, -0.2, 1.0],
        [-0.1, -0.1, 0.1, -0.1, -0.2, 1.0, -0.5, 0.1, -0.4]
    ];

    /** @const {Array<Array<number>>}*/ var s_constInMat4x3 = [
		[-0.9, 0.0, 0.6, 0.2, 0.9, -0.1, -0.3, -0.7, -0.1, 0.1, 1.0, 0.0],
        [0.5, 0.7, 0.7, 1.2, 1.1, 0.1, 1.0, -1.0, -0.2, -0.2, -0.3, -0.5]
	];

    /** @const {Array<Array<number>>}*/ var s_constInMat2x4 = [
		[-0.6, -1.1, -0.6, -0.6, -0.2, -0.6, -0.1, -0.1],
        [-1.2, -1.0, 0.7, -1.0, 0.7, 0.7, -0.4, -0.3]
	];

    /** @const {Array<Array<number>>}*/ var s_constInMat3x4 = [
        [0.6, -0.4, 1.2, 0.9, 0.8, 0.4, 1.1, 0.3, 0.5, -0.2, 0.0,  1.1],
		[-0.8, 1.2, -0.2, -1.1, -0.9, -0.5, -1.2, 1.0, 1.2, 0.1, -0.7, -0.5]
	];

    /** @const {Array<Array<number>>}*/ var s_constInMat4x4 = [
        [0.3, 0.9, -0.2, 1.0, -0.4, -0.6, 0.6, -1.0, -0.9, -0.1, 0.3, -0.2, -0.3, -0.9, 1.0, 0.1],
		[0.4, -0.7, -0.8, 0.7, -0.4, -0.8, 0.6, -0.3, 0.7, -1.0, 0.1, -0.3, 0.2, 0.6, 0.4, -1.0]
    ];

    // Operation info

    /**
     * @enum
     */
    es3fShaderMatrixTest.OperationType = {
    	OPERATIONTYPE_BINARY_OPERATOR: 0,
    	OPERATIONTYPE_BINARY_FUNCTION: 1,
    	OPERATIONTYPE_UNARY_PREFIX_OPERATOR: 2,
    	OPERATIONTYPE_UNARY_POSTFIX_OPERATOR: 3,
    	OPERATIONTYPE_UNARY_FUNCTION: 4,
    	OPERATIONTYPE_ASSIGNMENT: 5
    };

    /**
     * @param {es3fShaderMatrixTest.MatrixOp} op
     * @return {string}
     */
    es3fShaderMatrixTest.getOperationName = function(op) {
    	switch (op) {
    		case es3fShaderMatrixTest.MatrixOp.OP_ADD: return '+';
    		case es3fShaderMatrixTest.MatrixOp.OP_SUB: return '-';
    		case es3fShaderMatrixTest.MatrixOp.OP_MUL: return '*';
    		case es3fShaderMatrixTest.MatrixOp.OP_DIV: return '/';
    		case es3fShaderMatrixTest.MatrixOp.OP_COMP_MUL: return 'matrixCompMult';
    		case es3fShaderMatrixTest.MatrixOp.OP_OUTER_PRODUCT: return 'outerProduct';
    		case es3fShaderMatrixTest.MatrixOp.OP_TRANSPOSE: return 'transpose';
    		case es3fShaderMatrixTest.MatrixOp.OP_INVERSE: return 'inverse';
    		case es3fShaderMatrixTest.MatrixOp.OP_DETERMINANT: return 'determinant';
    		case es3fShaderMatrixTest.MatrixOp.OP_UNARY_PLUS: return '+';
    		case es3fShaderMatrixTest.MatrixOp.OP_NEGATION: return '-';
    		case es3fShaderMatrixTest.MatrixOp.OP_PRE_INCREMENT: return '++';
    		case es3fShaderMatrixTest.MatrixOp.OP_PRE_DECREMENT: return '--';
    		case es3fShaderMatrixTest.MatrixOp.OP_POST_INCREMENT: return '++';
    		case es3fShaderMatrixTest.MatrixOp.OP_POST_DECREMENT: return '--';
    		case es3fShaderMatrixTest.MatrixOp.OP_ADD_INTO: return '+=';
    		case es3fShaderMatrixTest.MatrixOp.OP_SUBTRACT_FROM: return '-=';
    		case es3fShaderMatrixTest.MatrixOp.OP_MULTIPLY_INTO: return '*=';
    		case es3fShaderMatrixTest.MatrixOp.OP_DIVIDE_INTO: return '/=';
    		default:
    			throw new Error('Error invalid Matrix Operation');
    	}
    };

    /**
     * @param {es3fShaderMatrixTest.MatrixOp} op
     * @return {es3fShaderMatrixTest.OperationType}
     */
    es3fShaderMatrixTest.getOperationType = function (op) {
    	switch (op)
    	{
    		case es3fShaderMatrixTest.MatrixOp.OP_ADD: return es3fShaderMatrixTest.OperationType.OPERATIONTYPE_BINARY_OPERATOR;
    		case es3fShaderMatrixTest.MatrixOp.OP_SUB: return es3fShaderMatrixTest.OperationType.OPERATIONTYPE_BINARY_OPERATOR;
    		case es3fShaderMatrixTest.MatrixOp.OP_MUL: return es3fShaderMatrixTest.OperationType.OPERATIONTYPE_BINARY_OPERATOR;
    		case es3fShaderMatrixTest.MatrixOp.OP_DIV: return es3fShaderMatrixTest.OperationType.OPERATIONTYPE_BINARY_OPERATOR;
    		case es3fShaderMatrixTest.MatrixOp.OP_COMP_MUL: return es3fShaderMatrixTest.OperationType.OPERATIONTYPE_BINARY_FUNCTION;
    		case es3fShaderMatrixTest.MatrixOp.OP_OUTER_PRODUCT: return es3fShaderMatrixTest.OperationType.OPERATIONTYPE_BINARY_FUNCTION;
    		case es3fShaderMatrixTest.MatrixOp.OP_TRANSPOSE: return es3fShaderMatrixTest.OperationType.OPERATIONTYPE_UNARY_FUNCTION;
    		case es3fShaderMatrixTest.MatrixOp.OP_INVERSE: return es3fShaderMatrixTest.OperationType.OPERATIONTYPE_UNARY_FUNCTION;
    		case es3fShaderMatrixTest.MatrixOp.OP_DETERMINANT:	return es3fShaderMatrixTest.OperationType.OPERATIONTYPE_UNARY_FUNCTION;
    		case es3fShaderMatrixTest.MatrixOp.OP_UNARY_PLUS: return es3fShaderMatrixTest.OperationType.OPERATIONTYPE_UNARY_PREFIX_OPERATOR;
    		case es3fShaderMatrixTest.MatrixOp.OP_NEGATION: return es3fShaderMatrixTest.OperationType.OPERATIONTYPE_UNARY_PREFIX_OPERATOR;
    		case es3fShaderMatrixTest.MatrixOp.OP_PRE_INCREMENT: return es3fShaderMatrixTest.OperationType.OPERATIONTYPE_UNARY_PREFIX_OPERATOR;
    		case es3fShaderMatrixTest.MatrixOp.OP_PRE_DECREMENT: return es3fShaderMatrixTest.OperationType.OPERATIONTYPE_UNARY_PREFIX_OPERATOR;
    		case es3fShaderMatrixTest.MatrixOp.OP_POST_INCREMENT: return es3fShaderMatrixTest.OperationType.OPERATIONTYPE_UNARY_POSTFIX_OPERATOR;
    		case es3fShaderMatrixTest.MatrixOp.OP_POST_DECREMENT: return es3fShaderMatrixTest.OperationType.OPERATIONTYPE_UNARY_POSTFIX_OPERATOR;
    		case es3fShaderMatrixTest.MatrixOp.OP_ADD_INTO: return es3fShaderMatrixTest.OperationType.OPERATIONTYPE_ASSIGNMENT;
    		case es3fShaderMatrixTest.MatrixOp.OP_SUBTRACT_FROM: return es3fShaderMatrixTest.OperationType.OPERATIONTYPE_ASSIGNMENT;
    		case es3fShaderMatrixTest.MatrixOp.OP_MULTIPLY_INTO: return es3fShaderMatrixTest.OperationType.OPERATIONTYPE_ASSIGNMENT;
    		case es3fShaderMatrixTest.MatrixOp.OP_DIVIDE_INTO: return es3fShaderMatrixTest.OperationType.OPERATIONTYPE_ASSIGNMENT;
    		default:
    			throw new Error('Error invalid Matrix Operation');
    	}
    };

    /**
     * @enum
     */
    es3fShaderMatrixTest.MatrixType = {
    	TESTMATRIXTYPE_DEFAULT: 0,
    	TESTMATRIXTYPE_NEGATED: 1,
    	TESTMATRIXTYPE_INCREMENTED: 2,
    	TESTMATRIXTYPE_DECREMENTED: 3,
    	TESTMATRIXTYPE_NEGATED_INCREMENTED: 4,
    	TESTMATRIXTYPE_INCREMENTED_LESS: 5
    };

    /**
     * @param {es3fShaderMatrixTest.MatrixOp} op
     * @return {es3fShaderMatrixTest.MatrixType}
     */
    es3fShaderMatrixTest.getOperationTestMatrixType = function (op) {
    	switch(op) {
    		case es3fShaderMatrixTest.MatrixOp.OP_ADD: return es3fShaderMatrixTest.MatrixType.TESTMATRIXTYPE_DEFAULT;
    		case es3fShaderMatrixTest.MatrixOp.OP_SUB: return es3fShaderMatrixTest.MatrixType.TESTMATRIXTYPE_DEFAULT;
    		case es3fShaderMatrixTest.MatrixOp.OP_MUL: return es3fShaderMatrixTest.MatrixType.TESTMATRIXTYPE_DEFAULT;
    		case es3fShaderMatrixTest.MatrixOp.OP_DIV: return es3fShaderMatrixTest.MatrixType.TESTMATRIXTYPE_DEFAULT;
    		case es3fShaderMatrixTest.MatrixOp.OP_COMP_MUL: return es3fShaderMatrixTest.MatrixType.TESTMATRIXTYPE_DEFAULT;
    		case es3fShaderMatrixTest.MatrixOp.OP_OUTER_PRODUCT: return es3fShaderMatrixTest.MatrixType.TESTMATRIXTYPE_DEFAULT;
    		case es3fShaderMatrixTest.MatrixOp.OP_TRANSPOSE: return es3fShaderMatrixTest.MatrixType.TESTMATRIXTYPE_DEFAULT;
    		case es3fShaderMatrixTest.MatrixOp.OP_INVERSE: return es3fShaderMatrixTest.MatrixType.TESTMATRIXTYPE_DEFAULT;
    		case es3fShaderMatrixTest.MatrixOp.OP_DETERMINANT: return es3fShaderMatrixTest.MatrixType.TESTMATRIXTYPE_DEFAULT;
    		case es3fShaderMatrixTest.MatrixOp.OP_UNARY_PLUS: return es3fShaderMatrixTest.MatrixType.TESTMATRIXTYPE_DECREMENTED;
    		case es3fShaderMatrixTest.MatrixOp.OP_NEGATION: return es3fShaderMatrixTest.MatrixType.TESTMATRIXTYPE_NEGATED_INCREMENTED;
    		case es3fShaderMatrixTest.MatrixOp.OP_PRE_INCREMENT: return es3fShaderMatrixTest.MatrixType.TESTMATRIXTYPE_NEGATED;
    		case es3fShaderMatrixTest.MatrixOp.OP_PRE_DECREMENT: return es3fShaderMatrixTest.MatrixType.TESTMATRIXTYPE_INCREMENTED;
    		case es3fShaderMatrixTest.MatrixOp.OP_POST_INCREMENT: return es3fShaderMatrixTest.MatrixType.TESTMATRIXTYPE_NEGATED;
    		case es3fShaderMatrixTest.MatrixOp.OP_POST_DECREMENT: return es3fShaderMatrixTest.MatrixType.TESTMATRIXTYPE_DEFAULT;
    		case es3fShaderMatrixTest.MatrixOp.OP_ADD_INTO: return es3fShaderMatrixTest.MatrixType.TESTMATRIXTYPE_DEFAULT;
    		case es3fShaderMatrixTest.MatrixOp.OP_SUBTRACT_FROM: return es3fShaderMatrixTest.MatrixType.TESTMATRIXTYPE_INCREMENTED_LESS;
    		case es3fShaderMatrixTest.MatrixOp.OP_MULTIPLY_INTO: return es3fShaderMatrixTest.MatrixType.TESTMATRIXTYPE_NEGATED;
    		case es3fShaderMatrixTest.MatrixOp.OP_DIVIDE_INTO: return es3fShaderMatrixTest.MatrixType.TESTMATRIXTYPE_DECREMENTED;
    		default:
    			throw new Error('Error invalid Matrix Operation');
    	}
    };

    /**
     * @param {es3fShaderMatrixTest.MatrixOp} op
     * @return {boolean}
     */
    es3fShaderMatrixTest.isOperationBinary = function (op) {
    	return es3fShaderMatrixTest.getOperationType(op) == es3fShaderMatrixTest.OperationType.OPERATIONTYPE_BINARY_OPERATOR ||
    	       es3fShaderMatrixTest.getOperationType(op) == es3fShaderMatrixTest.OperationType.OPERATIONTYPE_BINARY_FUNCTION ||
    	       es3fShaderMatrixTest.getOperationType(op) == es3fShaderMatrixTest.OperationType.OPERATIONTYPE_ASSIGNMENT;
    };

    /**
     * @param {es3fShaderMatrixTest.MatrixOp} op
     * @return {boolean}
     */
    es3fShaderMatrixTest.isOperationMatrixScalar = function (op) {
    	return op == es3fShaderMatrixTest.MatrixOp.OP_ADD ||
            op == es3fShaderMatrixTest.MatrixOp.OP_SUB ||
            op == es3fShaderMatrixTest.MatrixOp.OP_MUL ||
            op == es3fShaderMatrixTest.MatrixOp.OP_DIV;
    };

    /**
     * @param {es3fShaderMatrixTest.MatrixOp} op
     * @return {boolean}
     */
    es3fShaderMatrixTest.isOperationMatrixVector = function (op) {
        return op == es3fShaderMatrixTest.MatrixOp.OP_MUL;
    };


    /**
     * @param {es3fShaderMatrixTest.MatrixOp} op
     * @return {boolean}
     */
    es3fShaderMatrixTest.isOperationArithmeticMatrixMatrix = function (op) {
        return op == es3fShaderMatrixTest.MatrixOp.OP_MUL;
    };

    /**
     * @param {es3fShaderMatrixTest.MatrixOp} op
     * @return {boolean}
     */
    es3fShaderMatrixTest.isOperationComponentwiseMatrixMatrix = function (op) {
        return op == es3fShaderMatrixTest.MatrixOp.OP_ADD ||
            op == es3fShaderMatrixTest.MatrixOp.OP_SUB ||
            op == es3fShaderMatrixTest.MatrixOp.OP_MUL ||
            op == es3fShaderMatrixTest.MatrixOp.OP_DIV ||
            op == es3fShaderMatrixTest.MatrixOp.OP_COMP_MUL;
    };

    /**
     * @param {es3fShaderMatrixTest.MatrixOp} op
     * @return {boolean}
     */
    es3fShaderMatrixTest.isOperationVectorVector = function (op) {
    	return op == es3fShaderMatrixTest.MatrixOp.OP_OUTER_PRODUCT;
    };

    /**
     * @param {es3fShaderMatrixTest.MatrixOp} op
     * @return {boolean}
     */
    es3fShaderMatrixTest.isOperationUnaryAnyMatrix = function (op) {
    	return  op == es3fShaderMatrixTest.MatrixOp.OP_TRANSPOSE ||
            op == es3fShaderMatrixTest.MatrixOp.OP_UNARY_PLUS ||
    		op == es3fShaderMatrixTest.MatrixOp.OP_NEGATION ||
    		op == es3fShaderMatrixTest.MatrixOp.OP_PRE_INCREMENT ||
    		op == es3fShaderMatrixTest.MatrixOp.OP_PRE_DECREMENT ||
    		op == es3fShaderMatrixTest.MatrixOp.OP_POST_INCREMENT ||
    		op == es3fShaderMatrixTest.MatrixOp.OP_POST_DECREMENT;
    };

    /**
     * @param {es3fShaderMatrixTest.MatrixOp} op
     * @return {boolean}
     */
    es3fShaderMatrixTest.isOperationUnarySymmetricMatrix = function (op) {
    	return op == es3fShaderMatrixTest.MatrixOp.OP_INVERSE ||
            op == es3fShaderMatrixTest.MatrixOp.OP_DETERMINANT;
    };

    /**
     * @param {es3fShaderMatrixTest.MatrixOp} op
     * @return {boolean}
     */
    es3fShaderMatrixTest.isOperationValueModifying = function (op) {
    	return  op == es3fShaderMatrixTest.MatrixOp.OP_PRE_INCREMENT ||
    			op == es3fShaderMatrixTest.MatrixOp.OP_PRE_DECREMENT ||
    			op == es3fShaderMatrixTest.MatrixOp.OP_POST_INCREMENT ||
    			op == es3fShaderMatrixTest.MatrixOp.OP_POST_DECREMENT;
    };

    /**
     * @param {es3fShaderMatrixTest.MatrixOp} op
     * @return {boolean}
     */
    es3fShaderMatrixTest.isOperationAssignment = function(op) {
    	return  op == es3fShaderMatrixTest.MatrixOp.OP_ADD_INTO ||
    			op == es3fShaderMatrixTest.MatrixOp.OP_SUBTRACT_FROM ||
    			op == es3fShaderMatrixTest.MatrixOp.OP_MULTIPLY_INTO ||
    			op == es3fShaderMatrixTest.MatrixOp.OP_DIVIDE_INTO;
    };

    /**
     * @param {es3fShaderMatrixTest.MatrixOp} op
     * @return {boolean}
     */
    es3fShaderMatrixTest.isOperationAssignmentAnyMatrix = function(op) {
    	return  op == es3fShaderMatrixTest.MatrixOp.OP_ADD_INTO ||
    			op == es3fShaderMatrixTest.MatrixOp.OP_SUBTRACT_FROM ||
    			op == es3fShaderMatrixTest.MatrixOp.OP_DIVIDE_INTO;
    };

    /**
     * @param {es3fShaderMatrixTest.MatrixOp} op
     * @return {boolean}
     */
    es3fShaderMatrixTest.isOperationAssignmentSymmetricMatrix = function(op) {
    	return op == es3fShaderMatrixTest.MatrixOp.OP_MULTIPLY_INTO;
    };

    // Operation nature

    /**
     * @enum
     */
    es3fShaderMatrixTest.OperationNature = {
    	OPERATIONNATURE_PURE: 0,
    	OPERATIONNATURE_MUTATING: 1,
    	OPERATIONNATURE_ASSIGNMENT: 2
    };

    /**
     * @param {es3fShaderMatrixTest.MatrixOp} op
     * @return {es3fShaderMatrixTest.OperationNature}
     */
    es3fShaderMatrixTest.getOperationNature = function (op) {
    	if (es3fShaderMatrixTest.isOperationAssignment(op))
    		return es3fShaderMatrixTest.OperationNature.OPERATIONNATURE_ASSIGNMENT;
    	if (es3fShaderMatrixTest.isOperationValueModifying(op))
    		return es3fShaderMatrixTest.OperationNature.OPERATIONNATURE_MUTATING;
    	return es3fShaderMatrixTest.OperationNature.OPERATIONNATURE_PURE;
    };

    // Input value loader.
    /**
     * @param {es3fShaderMatrixTest.InputType} inputType
     * @param {gluShaderUtil.DataType} typeFormat
     * @param {glsShaderRenderCase.ShaderEvalContext} evalCtx
     * @param {number} inputNdx
     * @return {Array<number>|tcuMatrix.Matrix|number}
     */
    es3fShaderMatrixTest.getInputValue = function (inputType, typeFormat, evalCtx, inputNdx) {
        if (inputType == es3fShaderMatrixTest.InputType.INPUTTYPE_CONST) {
            switch (typeFormat) {
                case gluShaderUtil.DataType.FLOAT:
                    return s_constInFloat[inputNdx];
                case gluShaderUtil.DataType.FLOAT_VEC2:
                    return s_constInVec2[inputNdx];
                case gluShaderUtil.DataType.FLOAT_VEC3:
                    return s_constInVec3[inputNdx];
                case gluShaderUtil.DataType.FLOAT_VEC4:
                    return s_constInVec4[inputNdx];
                case gluShaderUtil.DataType.FLOAT_MAT2:
                    return tcuMatrix.matrixFromDataArray(2, 2, s_constInMat2x2[inputNdx]);
                case gluShaderUtil.DataType.FLOAT_MAT2X3:
                    return tcuMatrix.matrixFromDataArray(3, 2, s_constInMat2x3[inputNdx]);
                case gluShaderUtil.DataType.FLOAT_MAT2X4:
                    return tcuMatrix.matrixFromDataArray(4, 2, s_constInMat2x4[inputNdx]);
                case gluShaderUtil.DataType.FLOAT_MAT3X2:
                    return tcuMatrix.matrixFromDataArray(2, 3, s_constInMat3x2[inputNdx]);
                case gluShaderUtil.DataType.FLOAT_MAT3:
                    return tcuMatrix.matrixFromDataArray(3, 3, s_constInMat3x3[inputNdx]);
                case gluShaderUtil.DataType.FLOAT_MAT3X4:
                    return tcuMatrix.matrixFromDataArray(4, 3, s_constInMat3x4[inputNdx]);
                case gluShaderUtil.DataType.FLOAT_MAT4X2:
                    return tcuMatrix.matrixFromDataArray(2, 4, s_constInMat4x2[inputNdx]);
                case gluShaderUtil.DataType.FLOAT_MAT4X3:
                    return tcuMatrix.matrixFromDataArray(3, 4, s_constInMat4x3[inputNdx]);
                case gluShaderUtil.DataType.FLOAT_MAT4:
                    return tcuMatrix.matrixFromDataArray(4, 4, s_constInMat4x4[inputNdx]);
            }
        } else if (inputType == es3fShaderMatrixTest.InputType.INPUTTYPE_DYNAMIC) {
            switch (typeFormat) {
                case gluShaderUtil.DataType.FLOAT:
                    return evalCtx.coords[0];
                case gluShaderUtil.DataType.FLOAT_VEC2:
                    return deMath.swizzle(evalCtx.coords, [0, 1]);
                case gluShaderUtil.DataType.FLOAT_VEC3:
                    return deMath.swizzle(evalCtx.coords, [0, 1, 2]);
                case gluShaderUtil.DataType.FLOAT_VEC4:
                    return deMath.swizzle(evalCtx.coords, [0, 1, 2, 3]);
                case gluShaderUtil.DataType.FLOAT_MAT2:
                    var m = new tcuMatrix.Matrix(2, 2);
                    m.setCol(0, deMath.swizzle(evalCtx.in_[0], [0, 1]));
                    m.setCol(1, deMath.swizzle(evalCtx.in_[1], [0, 1]));
                    return m;
                case gluShaderUtil.DataType.FLOAT_MAT2X3:
                    var m = new tcuMatrix.Matrix(3, 2);
                    m.setCol(0, deMath.swizzle(evalCtx.in_[0], [0, 1, 2]));
                    m.setCol(1, deMath.swizzle(evalCtx.in_[1], [0, 1, 2]));
                    return m;
                case gluShaderUtil.DataType.FLOAT_MAT2X4:
                    var m = new tcuMatrix.Matrix(4, 2);
                    m.setCol(0, deMath.swizzle(evalCtx.in_[0], [0, 1, 2, 3]));
                    m.setCol(1, deMath.swizzle(evalCtx.in_[1], [0, 1, 2, 3]));
                    return m;
                case gluShaderUtil.DataType.FLOAT_MAT3X2:
                    var m = new tcuMatrix.Matrix(2, 3);
                    m.setCol(0, deMath.swizzle(evalCtx.in_[0], [0, 1]));
                    m.setCol(1, deMath.swizzle(evalCtx.in_[1], [0, 1]));
                    m.setCol(2, deMath.swizzle(evalCtx.in_[2], [0, 1]));
                    return m;
                case gluShaderUtil.DataType.FLOAT_MAT3:
                    var m = new tcuMatrix.Matrix(3, 3);
                    m.setCol(0, deMath.swizzle(evalCtx.in_[0], [0, 1, 2]));
                    m.setCol(1, deMath.swizzle(evalCtx.in_[1], [0, 1, 2]));
                    m.setCol(2, deMath.swizzle(evalCtx.in_[2], [0, 1, 2]));
                    return m;
                case gluShaderUtil.DataType.FLOAT_MAT3X4:
                    var m = new tcuMatrix.Matrix(4, 3);
                    m.setCol(0, evalCtx.in_[0]);
                    m.setCol(1, evalCtx.in_[1]);
                    m.setCol(2, evalCtx.in_[2]);
                    return m;
                case gluShaderUtil.DataType.FLOAT_MAT4X2:
                    var m = new tcuMatrix.Matrix(2, 4);
                    m.setCol(0, deMath.swizzle(evalCtx.in_[0], [0, 1]));
                    m.setCol(1, deMath.swizzle(evalCtx.in_[1], [0, 1]));
                    m.setCol(2, deMath.swizzle(evalCtx.in_[2], [0, 1]));
                    m.setCol(3, deMath.swizzle(evalCtx.in_[3], [0, 1]));
                    return m;
                case gluShaderUtil.DataType.FLOAT_MAT4X3:
                    var m = new tcuMatrix.Matrix(3, 4);
                    m.setCol(0, deMath.swizzle(evalCtx.in_[0], [0, 1, 2]));
                    m.setCol(1, deMath.swizzle(evalCtx.in_[1], [0, 1, 2]));
                    m.setCol(2, deMath.swizzle(evalCtx.in_[2], [0, 1, 2]));
                    m.setCol(3, deMath.swizzle(evalCtx.in_[3], [0, 1, 2]));
                    return m;
                case gluShaderUtil.DataType.FLOAT_MAT4:
                    var m = new tcuMatrix.Matrix(4, 4);
                    m.setCol(0, evalCtx.in_[0]);
                    m.setCol(1, evalCtx.in_[1]);
                    m.setCol(2, evalCtx.in_[2]);
                    m.setCol(3, evalCtx.in_[3]);
                    return m;
            }
        }
        throw new Error('Invalid input type');
    };

    /**
     * @param {Array<number>} value
     * @return {Array<number>}
     */
    es3fShaderMatrixTest.reduceVecToVec3 = function (value) {
        if (value.length == 3) {
            return value;
        } else if (value.length == 2) {
            return deMath.swizzle(value, [0, 1, 0])
        } else {
            return [value[0], value[1], value[2] + value[3]];
        }
    };

    /**
     * @param {tcuMatrix.Matrix} value
     * @return {Array<number>}
     */
    es3fShaderMatrixTest.reduceMatToVec3 = function (value) {
        if (value.cols == 2) {
            if (value.rows == 2) {
                // mat2
                return [value.get(0, 0), value.get(0, 1), value.get(1, 0) + value.get(1, 1)];
            } else if (value.rows == 3){
                //mat2x3
                return deMath.add(value.getColumn(0), value.getColumn(1));
            } else {
                //mat2x4
                return deMath.add(deMath.swizzle(value.getColumn(0), [0, 1, 2]), deMath.swizzle(value.getColumn(1), [1, 2, 3]));
            }
        } else if (value.cols == 3) {
            if (value.rows == 2) {
                return [value.get(0, 0) + value.get(1, 0), value.get(0, 1) + value.get(1, 1), value.get(0, 2) + value.get(1, 2)];
            } else if (value.rows == 3) {
                return deMath.add(deMath.add(value.getColumn(0), value.getColumn(1)), value.getColumn(2));
            } else {
                return deMath.add(deMath.add(deMath.swizzle(value.getColumn(0), [0, 1, 2]), deMath.swizzle(value.getColumn(1), [1, 2, 3])), deMath.swizzle(value.getColumn(2), [2, 3, 0]))
            }
        } else {
            if (value.rows == 2) {
                return [value.get(0, 0) + value.get(1, 0) + value.get(0, 3), value.get(0, 1) + value.get(1, 1) + value.get(1, 3), value.get(0, 2) + value.get(1, 2)];
            } else if (value.rows == 3) {
                return deMath.add(deMath.add(deMath.add(value.getColumn(0), value.getColumn(1)), value.getColumn(2)), value.getColumn(3));
            } else {
                return deMath.add(deMath.add(deMath.add(deMath.swizzle(value.getColumn(0), [0, 1, 2]), deMath.swizzle(value.getColumn(1), [1, 2, 3])), deMath.swizzle(value.getColumn(2), [2, 3, 0])), deMath.swizzle(value.getColumn(3), [3, 0, 1]));
            }
        }
    };

    /**
     * @param {Array<number>|tcuMatrix.Matrix|number} value
     * @return {Array<number>}
     */
    es3fShaderMatrixTest.reduceToVec3 = function (value) {
        if (value instanceof tcuMatrix.Matrix)
            return es3fShaderMatrixTest.reduceMatToVec3(value);
        else if (value instanceof Array)
            return es3fShaderMatrixTest.reduceVecToVec3(value);
        else
            throw new Error('Impossible case');
    };

    es3fShaderMatrixTest.add = function (a, b) {
        if (a instanceof tcuMatrix.Matrix) {
            if (b instanceof tcuMatrix.Matrix)
                return tcuMatrix.add(a, b);
            else if (b instanceof Array)
                throw new Error('Unimplemented');
            else
                return tcuMatrix.addMatScal(a, b);
        }
        else {
            if (b instanceof tcuMatrix.Matrix)
                throw new Error('Unimplemented');
            else
                return deMath.add(a, b);
        }
    };

    es3fShaderMatrixTest.subtract = function (a, b) {
        if (a instanceof tcuMatrix.Matrix) {
            if (b instanceof tcuMatrix.Matrix)
                return tcuMatrix.subtract(a, b);
            else if (b instanceof Array)
                throw new Error('Unimplemented');
            else
                return tcuMatrix.subtractMatScal(a, b);
        }
        else {
            if (b instanceof tcuMatrix.Matrix)
                throw new Error('Unimplemented');
            else
                return deMath.subtract(a, b);
        }
    };

    es3fShaderMatrixTest.multiply = function (a, b) {
        if (a instanceof tcuMatrix.Matrix) {
            if (b instanceof tcuMatrix.Matrix)
                return tcuMatrix.multiply(a, b);
            else if (b instanceof Array)
                return tcuMatrix.multiplyMatVec(a, b);
            else
                return tcuMatrix.multiplyMatScal(a, b);
        } else {
            if (b instanceof tcuMatrix.Matrix)
                return tcuMatrix.multiplyVecMat(a, b);
            else
                return deMath.multiply(a, b);
        }
    };

    es3fShaderMatrixTest.divide = function (a, b) {
        if (a instanceof tcuMatrix.Matrix) {
            if (b instanceof tcuMatrix.Matrix)
                return tcuMatrix.divide(a, b);
            else if (b instanceof Array)
                throw new Error('Unimplemented');
            else
                return tcuMatrix.divideMatScal(a, b);
        }
        else {
            if (b instanceof tcuMatrix.Matrix)
                throw new Error('Unimplemented');
            else
                return deMath.divide(a, b);
        }
    };


    /**
     * @param {tcuMatrix.Matrix} a
     * @param {tcuMatrix.Matrix} b
     * @return {tcuMatrix.Matrix}
     */
    es3fShaderMatrixTest.matrixCompMult = function (a, b) {
        /** @type {tcuMatrix.Matrix} */ var retVal = new tcuMatrix.Matrix(a.rows, a.cols);

        for (var r = 0; r < a.rows; ++r) {
            for (var c = 0; c < a.cols; ++c) {
                retVal.set(r, c, a.get(r, c) * b.get(r, c));
            }
        }
        return retVal;
    };

    /**
     * @param {tcuMatrix.Matrix} mat
     * @return {tcuMatrix.Matrix}
     */
    es3fShaderMatrixTest.transpose = function (mat) {
        /** @type {tcuMatrix.Matrix} */ var retVal = new tcuMatrix.Matrix(mat.cols, mat.rows);

        for (var r = 0; r < mat.rows; ++r) {
            for (var c = 0; c < mat.cols; ++c) {
                retVal.set(c, r, mat.get(r, c));
            }
        }

        return retVal;
    };

    /**
     * @param {tcuMatrix.Matrix} mat
     * @return {number}
     */
    es3fShaderMatrixTest.determinantMat2 = function (mat) {
        return mat.get(0, 0) * mat.get(1, 1) - mat.get(1, 0) * mat.get(0,1);
    };

    /**
     * @param {tcuMatrix.Matrix} mat
     * @return {number}
     */
    es3fShaderMatrixTest.determinantMat3 = function (mat) {
        return	+ mat.get(0, 0) * mat.get(1, 1) * mat.get(2, 2)
			+ mat.get(0, 1) * mat.get(1, 2) * mat.get(2, 0)
			+ mat.get(0, 2) * mat.get(1, 0) * mat.get(2, 1)
			- mat.get(0, 0) * mat.get(1, 2) * mat.get(2, 1)
			- mat.get(0, 1) * mat.get(1, 0) * mat.get(2, 2)
			- mat.get(0, 2) * mat.get(1, 1) * mat.get(2, 0);
    };

    /**
     * @param {tcuMatrix.Matrix} mat
     * @return {number}
     */
    es3fShaderMatrixTest.determinantMat4 = function (mat) {
        /** @type {Array<Array<number>>} */ var minorMatrices = [
            [
    			mat.get(1, 1),	mat.get(2, 1),	mat.get(3, 1),
    			mat.get(1, 2),	mat.get(2, 2),	mat.get(3, 2),
    			mat.get(1, 3),	mat.get(2, 3),	mat.get(3, 3)
    		],
    		[
    			mat.get(1, 0),	mat.get(2, 0),	mat.get(3, 0),
    			mat.get(1, 2),	mat.get(2, 2),	mat.get(3, 2),
    			mat.get(1, 3),	mat.get(2, 3),	mat.get(3, 3)
    		],
    		[
    			mat.get(1, 0),	mat.get(2, 0),	mat.get(3, 0),
    			mat.get(1, 1),	mat.get(2, 1),	mat.get(3, 1),
    			mat.get(1, 3),	mat.get(2, 3),	mat.get(3, 3)
    		],
    		[
    			mat.get(1, 0),	mat.get(2, 0),	mat.get(3, 0),
    			mat.get(1, 1),	mat.get(2, 1),	mat.get(3, 1),
    			mat.get(1, 2),	mat.get(2, 2),	mat.get(3, 2)
    		]
        ];

    	return	+ mat.get(0, 0) * es3fShaderMatrixTest.determinant(tcuMatrix.matrixFromDataArray(3, 3, minorMatrices[0]))
    			- mat.get(0, 1) * es3fShaderMatrixTest.determinant(tcuMatrix.matrixFromDataArray(3, 3, minorMatrices[1]))
    			+ mat.get(0, 2) * es3fShaderMatrixTest.determinant(tcuMatrix.matrixFromDataArray(3, 3, minorMatrices[2]))
    			- mat.get(0, 3) * es3fShaderMatrixTest.determinant(tcuMatrix.matrixFromDataArray(3, 3, minorMatrices[3]));
    };

    /**
     * @param {tcuMatrix.Matrix} mat
     * @return {number}
     */
    es3fShaderMatrixTest.determinant = function (mat) {
        if (mat.rows == 2) {
            return es3fShaderMatrixTest.determinantMat2(mat);
        } else if (mat.rows == 3) {
            return es3fShaderMatrixTest.determinantMat3(mat);
        } else {
            return es3fShaderMatrixTest.determinantMat4(mat);
        }
    };

    /**
     * @param {tcuMatrix.Matrix} mat
     * @return {tcuMatrix.Matrix}
     */
    es3fShaderMatrixTest.inverseMat2 = function (mat) {
        /** @type {number} */ var det = es3fShaderMatrixTest.determinant(mat);
        /** @type {tcuMatrix.Matrix} */ var retVal = new tcuMatrix.Mat2();

        if (det == 0.0) {
            throw new Error('Wrong determinant')
        }

        retVal.set(0, 0, mat.get(1, 1) / det);
        retVal.set(0, 1, -mat.get(0, 1) / det);
        retVal.set(1, 0, -mat.get(1, 0) / det);
        retVal.set(1, 1, mat.get(0, 0) / det);

        return retVal;
    };

    /**
     * @param {tcuMatrix.Matrix} mat
     * @return {tcuMatrix.Matrix}
     */
    es3fShaderMatrixTest.inverseMat3 = function (mat) {
        if (es3fShaderMatrixTest.determinant(mat) == 0.0) {
            throw new Error('Wrong determinant')
        }

    	/** @type {Array<number>} */ var areaA = [mat.get(0, 0), mat.get(0, 1), mat.get(1, 0), mat.get(1,1)];
        /** @type {Array<number>} */ var areaB = [mat.get(0, 2), mat.get(1, 2)];
        /** @type {Array<number>} */ var areaC = [mat.get(2, 0), mat.get(2, 1)];
        /** @type {Array<number>} */ var areaD = [mat.get(2,2)];

    	/** @type {tcuMatrix.Matrix} */ var	invA = es3fShaderMatrixTest.inverse(tcuMatrix.matrixFromDataArray(2, 2, areaA));
    	/** @type {tcuMatrix.Matrix} */ var	matB = tcuMatrix.matrixFromDataArray(2, 1, areaB);
    	/** @type {tcuMatrix.Matrix} */ var	matC = tcuMatrix.matrixFromDataArray(1, 2, areaC);
    	/** @type {tcuMatrix.Matrix} */ var	matD = tcuMatrix.matrixFromDataArray(1, 1, areaD);

        /** @type {tcuMatrix.Matrix} */ var tmp = tcuMatrix.subtract(matD, tcuMatrix.multiply(matC, tcuMatrix.multiply(invA, matB)));
    	/** @type {number} */ var schurComplement = 1.0 / tmp.get(0, 0);
    	/** @type {tcuMatrix.Matrix} */ var	zeroMat = new tcuMatrix.Matrix(2, 2, 0);

    	/** @type {tcuMatrix.Matrix} */ var	blockA = tcuMatrix.add(invA, tcuMatrix.multiply(tcuMatrix.multiply(invA, tcuMatrix.multiply(tcuMatrix.multiplyMatScal(matB, schurComplement), matC)), invA));
    	/** @type {tcuMatrix.Matrix} */ var	blockB = tcuMatrix.multiplyMatScal(tcuMatrix.multiply(tcuMatrix.subtract(zeroMat, invA), matB), schurComplement);
    	/** @type {tcuMatrix.Matrix} */ var	blockC = tcuMatrix.multiply(matC, tcuMatrix.multiplyMatScal(invA, - schurComplement));
    	/** @type {number} */ var blockD = schurComplement;

    	/** @type {Array<number>} */ var result = [
    		blockA.get(0, 0), blockA.get(0, 1), blockB.get(0, 0),
            blockA.get(1, 0), blockA.get(1, 1), blockB.get(1, 0),
            blockC.get(0, 0), blockC.get(0, 1),	blockD
    	];

    	return tcuMatrix.matrixFromDataArray(3, 3, result);
    }

    /**
     * @param {tcuMatrix.Matrix} mat
     * @return {tcuMatrix.Matrix}
     */
    es3fShaderMatrixTest.inverseMat4 = function (mat) {
        // Blockwise inversion
        if (es3fShaderMatrixTest.determinant(mat) == 0.0) {
            throw new Error('Wrong determinant')
        }

    	/** @type {Array<number>} */ var areaA = [
    		mat.get(0, 0),	mat.get(0, 1),
    		mat.get(1, 0),	mat.get(1, 1)
    	];
    	/** @type {Array<number>} */ var areaB = [
    		mat.get(0, 2),	mat.get(0, 3),
    		mat.get(1, 2),	mat.get(1, 3)
    	];
    	/** @type {Array<number>} */ var areaC = [
    		mat.get(2, 0),	mat.get(2, 1),
    		mat.get(3, 0),	mat.get(3, 1)
    	];
    	/** @type {Array<number>} */ var areaD = [
    		mat.get(2, 2),	mat.get(2, 3),
    		mat.get(3, 2),	mat.get(3, 3)
    	];

    	/** @type {tcuMatrix.Matrix} */ var	invA = es3fShaderMatrixTest.inverse(tcuMatrix.matrixFromDataArray(2, 2, areaA));
    	/** @type {tcuMatrix.Matrix} */ var	matB = tcuMatrix.matrixFromDataArray(2, 2, areaB);
    	/** @type {tcuMatrix.Matrix} */ var	matC = tcuMatrix.matrixFromDataArray(2, 2, areaC);
    	/** @type {tcuMatrix.Matrix} */ var	matD = tcuMatrix.matrixFromDataArray(2, 2, areaD);

    	/** @type {tcuMatrix.Matrix} */ var	schurComplement = es3fShaderMatrixTest.inverse(tcuMatrix.subtract(matD, (tcuMatrix.multiply(matC, tcuMatrix.multiply(invA, matB)))));
    	/** @type {tcuMatrix.Matrix} */ var	zeroMat = new tcuMatrix.Matrix(2, 2, 0);

    	/** @type {tcuMatrix.Matrix} */ var	blockA = tcuMatrix.add(invA, tcuMatrix.multiply(tcuMatrix.multiply(tcuMatrix.multiply(tcuMatrix.multiply(invA, matB), schurComplement), matC), invA));
    	/** @type {tcuMatrix.Matrix} */ var	blockB = tcuMatrix.multiply(tcuMatrix.multiply(tcuMatrix.subtract(zeroMat, invA), matB), schurComplement);
    	/** @type {tcuMatrix.Matrix} */ var	blockC = tcuMatrix.multiply(tcuMatrix.multiply(tcuMatrix.subtract(zeroMat, schurComplement),matC), invA);
    	/** @type {tcuMatrix.Matrix} */ var	blockD = schurComplement;

    	/** @type {Array<number>} */ var result = [
    		blockA.get(0, 0),	blockA.get(0, 1),	blockB.get(0, 0),	blockB.get(0, 1),
    		blockA.get(1, 0),	blockA.get(1, 1),	blockB.get(1, 0),	blockB.get(1, 1),
    		blockC.get(0, 0),	blockC.get(0, 1),	blockD.get(0, 0),	blockD.get(0, 1),
    		blockC.get(1, 0),	blockC.get(1, 1),	blockD.get(1, 0),	blockD.get(1, 1)
    	];

    	return tcuMatrix.matrixFromDataArray(4, 4, result);
    };

    /**
     * @param {tcuMatrix.Matrix} mat
     * @return {tcuMatrix.Matrix}
     */
    es3fShaderMatrixTest.inverse = function (mat) {
        if (mat.cols == 2) {
            return es3fShaderMatrixTest.inverseMat2(mat)
        } else if (mat.cols == 3) {
            return es3fShaderMatrixTest.inverseMat3(mat)
        } else {
            return es3fShaderMatrixTest.inverseMat4(mat)
        }
    };

    /**
     * @param {tcuMatrix.Matrix} mat
     * @return {tcuMatrix.Matrix}
     */
    es3fShaderMatrixTest.negate = function (mat) {
        /** @type {tcuMatrix.Matrix} */ var retVal = new tcuMatrix.Matrix(mat.rows, mat.cols);

    	for (var r = 0; r < mat.rows; ++r)
    		for (var c = 0; c < mat.cols; ++c)
    			retVal.set(r,c, -mat.get(r, c));

    	return retVal;
    };

    /**
     * @param {tcuMatrix.Matrix} mat
     * @return {tcuMatrix.Matrix}
     */
    es3fShaderMatrixTest.increment = function (mat) {
        /** @type {tcuMatrix.Matrix} */ var retVal = new tcuMatrix.Matrix(mat.rows, mat.cols);

    	for (var r = 0; r < mat.rows; ++r)
    		for (var c = 0; c < mat.cols; ++c)
    			retVal.set(r,c, mat.get(r, c) + 1.0);

    	return retVal;
    };

    /**
     * @param {tcuMatrix.Matrix} mat
     * @return {tcuMatrix.Matrix}
     */
    es3fShaderMatrixTest.decrement = function (mat) {
        /** @type {tcuMatrix.Matrix} */ var retVal = new tcuMatrix.Matrix(mat.rows, mat.cols);

    	for (var r = 0; r < mat.rows; ++r)
    		for (var c = 0; c < mat.cols; ++c)
    			retVal.set(r,c, mat.get(r, c) - 1.0);

    	return retVal;
    };

    /**
     * @param {Array<number>} a
     * @param {Array<number>} b
     * @return {tcuMatrix.Matrix}
     */
    es3fShaderMatrixTest.outerProduct = function (a, b) {
        /** @type {tcuMatrix.Matrix} */ var retVal = new tcuMatrix.Matrix(b.length, a.length);

        for (var r = 0; r < b.length; ++r) {
            for (var c = 0; c < a.length; ++c) {
                retVal.set(r, c, a[c] * b[r]);
            }
        }

        return es3fShaderMatrixTest.transpose(retVal);
    };

    /**
     * @enum
     */
    es3fShaderMatrixTest.InputType = {
    	INPUTTYPE_CONST: 0,
    	INPUTTYPE_UNIFORM: 1,
    	INPUTTYPE_DYNAMIC: 2
    };

    /**
     * @enum
     */
    es3fShaderMatrixTest.MatrixOp = {
    	OP_ADD: 0,
    	OP_SUB: 1,
    	OP_MUL: 2,
    	OP_DIV: 3,
    	OP_COMP_MUL: 4,
    	OP_OUTER_PRODUCT: 5,
    	OP_TRANSPOSE: 6,
    	OP_INVERSE: 7,
    	OP_DETERMINANT: 8,
    	OP_UNARY_PLUS: 9,
    	OP_NEGATION: 10,
    	OP_PRE_INCREMENT: 11,
    	OP_PRE_DECREMENT: 12,
    	OP_POST_INCREMENT: 13,
    	OP_POST_DECREMENT: 14,
    	OP_ADD_INTO: 15,
    	OP_SUBTRACT_FROM: 16,
    	OP_MULTIPLY_INTO: 17,
    	OP_DIVIDE_INTO: 18,
    	OP_LAST: 19
    };

    /**
     * @constructor
     * @param {es3fShaderMatrixTest.InputType=} inputType_
     * @param {gluShaderUtil.DataType=} dataType_
     * @param {gluShaderUtil.precision=} precision_
     * @struct
     */
    es3fShaderMatrixTest.ShaderInput = function (inputType_, dataType_, precision_){
        this.inputType = inputType_ || es3fShaderMatrixTest.InputType.INPUTTYPE_CONST;
        this.dataType = dataType_ || gluShaderUtil.DataType.INVALID;
        this.precision = precision_ || gluShaderUtil.precision.PRECISION_LOWP;
    };

    /**
     * @param {es3fShaderMatrixTest.ShaderInput} in0
     * @param {es3fShaderMatrixTest.ShaderInput} in1
     * @param {es3fShaderMatrixTest.MatrixOp} op
     */
    es3fShaderMatrixTest.getEvalFunc = function (in0, in1, op) {
        var setColor = function(evalCtx, src) {
            for (var i = 0; i < 3; i++)
                evalCtx.color[i] = src[i];
        };
        switch(op){
            case es3fShaderMatrixTest.MatrixOp.OP_ADD:
                return function (evalCtx) {
                    var in0_ = in0.inputType == es3fShaderMatrixTest.InputType.INPUTTYPE_DYNAMIC ?
                        es3fShaderMatrixTest.getInputValue(in0.inputType, in0.dataType, evalCtx, 0)
                        : es3fShaderMatrixTest.getInputValue(es3fShaderMatrixTest.InputType.INPUTTYPE_CONST, in0.dataType, evalCtx, 0);
                    var in1_ = in1.inputType == es3fShaderMatrixTest.InputType.INPUTTYPE_DYNAMIC ?
                        es3fShaderMatrixTest.getInputValue(in1.inputType, in1.dataType, evalCtx, 1)
                        : es3fShaderMatrixTest.getInputValue(es3fShaderMatrixTest.InputType.INPUTTYPE_CONST, in1.dataType, evalCtx, 1);

                    setColor(evalCtx, es3fShaderMatrixTest.reduceToVec3(es3fShaderMatrixTest.add(in0_, in1_)));
                };
        	case es3fShaderMatrixTest.MatrixOp.OP_SUB:
                return function (evalCtx) {
                    var in0_ = in0.inputType == es3fShaderMatrixTest.InputType.INPUTTYPE_DYNAMIC ?
                        es3fShaderMatrixTest.getInputValue(in0.inputType, in0.dataType, evalCtx, 0)
                        : es3fShaderMatrixTest.getInputValue(es3fShaderMatrixTest.InputType.INPUTTYPE_CONST, in0.dataType, evalCtx, 0);
                    var in1_ = in1.inputType == es3fShaderMatrixTest.InputType.INPUTTYPE_DYNAMIC ?
                        es3fShaderMatrixTest.getInputValue(in1.inputType, in1.dataType, evalCtx, 1)
                        : es3fShaderMatrixTest.getInputValue(es3fShaderMatrixTest.InputType.INPUTTYPE_CONST, in1.dataType, evalCtx, 1);

                    setColor(evalCtx, es3fShaderMatrixTest.reduceToVec3(es3fShaderMatrixTest.subtract(in0_, in1_)));
                };
        	case es3fShaderMatrixTest.MatrixOp.OP_MUL:
                return function (evalCtx) {
                    var in0_ = in0.inputType == es3fShaderMatrixTest.InputType.INPUTTYPE_DYNAMIC ?
                        es3fShaderMatrixTest.getInputValue(in0.inputType, in0.dataType, evalCtx, 0)
                        : es3fShaderMatrixTest.getInputValue(es3fShaderMatrixTest.InputType.INPUTTYPE_CONST, in0.dataType, evalCtx, 0);
                    var in1_ = in1.inputType == es3fShaderMatrixTest.InputType.INPUTTYPE_DYNAMIC ?
                        es3fShaderMatrixTest.getInputValue(in1.inputType, in1.dataType, evalCtx, 1)
                        : es3fShaderMatrixTest.getInputValue(es3fShaderMatrixTest.InputType.INPUTTYPE_CONST, in1.dataType, evalCtx, 1);

                    setColor(evalCtx, es3fShaderMatrixTest.reduceToVec3(es3fShaderMatrixTest.multiply(in0_, in1_)));
                };
        	case es3fShaderMatrixTest.MatrixOp.OP_DIV:
                return function (evalCtx) {
                    var in0_ = in0.inputType == es3fShaderMatrixTest.InputType.INPUTTYPE_DYNAMIC ?
                        es3fShaderMatrixTest.getInputValue(in0.inputType, in0.dataType, evalCtx, 0)
                        : es3fShaderMatrixTest.getInputValue(es3fShaderMatrixTest.InputType.INPUTTYPE_CONST, in0.dataType, evalCtx, 0);
                    var in1_ = in1.inputType == es3fShaderMatrixTest.InputType.INPUTTYPE_DYNAMIC ?
                        es3fShaderMatrixTest.getInputValue(in1.inputType, in1.dataType, evalCtx, 1)
                        : es3fShaderMatrixTest.getInputValue(es3fShaderMatrixTest.InputType.INPUTTYPE_CONST, in1.dataType, evalCtx, 1);

                    setColor(evalCtx, es3fShaderMatrixTest.reduceToVec3(es3fShaderMatrixTest.divide(in0_, in1_)));
                };
        	case es3fShaderMatrixTest.MatrixOp.OP_COMP_MUL:
                return function (evalCtx) {
                    var in0_ = in0.inputType == es3fShaderMatrixTest.InputType.INPUTTYPE_DYNAMIC ?
                        es3fShaderMatrixTest.getInputValue(in0.inputType, in0.dataType, evalCtx, 0)
                        : es3fShaderMatrixTest.getInputValue(es3fShaderMatrixTest.InputType.INPUTTYPE_CONST, in0.dataType, evalCtx, 0);
                    var in1_ = in1.inputType == es3fShaderMatrixTest.InputType.INPUTTYPE_DYNAMIC ?
                        es3fShaderMatrixTest.getInputValue(in1.inputType, in1.dataType, evalCtx, 1)
                        : es3fShaderMatrixTest.getInputValue(es3fShaderMatrixTest.InputType.INPUTTYPE_CONST, in1.dataType, evalCtx, 1);

                    setColor(evalCtx, es3fShaderMatrixTest.reduceToVec3(es3fShaderMatrixTest.matrixCompMult(/** @type {tcuMatrix.Matrix} */(in0_), /** @type {tcuMatrix.Matrix} */(in1_))));
                };
        	case es3fShaderMatrixTest.MatrixOp.OP_OUTER_PRODUCT:
                return function (evalCtx) {
                    var in0_ = in0.inputType == es3fShaderMatrixTest.InputType.INPUTTYPE_DYNAMIC ?
                        es3fShaderMatrixTest.getInputValue(in0.inputType, in0.dataType, evalCtx, 0)
                        : es3fShaderMatrixTest.getInputValue(es3fShaderMatrixTest.InputType.INPUTTYPE_CONST, in0.dataType, evalCtx, 0);
                    var in1_ = in1.inputType == es3fShaderMatrixTest.InputType.INPUTTYPE_DYNAMIC ?
                        es3fShaderMatrixTest.getInputValue(in1.inputType, in1.dataType, evalCtx, 1)
                        : es3fShaderMatrixTest.getInputValue(es3fShaderMatrixTest.InputType.INPUTTYPE_CONST, in1.dataType, evalCtx, 1);

                    setColor(evalCtx, es3fShaderMatrixTest.reduceToVec3(es3fShaderMatrixTest.outerProduct(/** @type {Array<number>} */(in0_), /** @type {Array<number>} */(in1_))));
                };
        	case es3fShaderMatrixTest.MatrixOp.OP_TRANSPOSE:
                return function (evalCtx) {
                    var in0_ = in0.inputType == es3fShaderMatrixTest.InputType.INPUTTYPE_DYNAMIC ?
                        es3fShaderMatrixTest.getInputValue(in0.inputType, in0.dataType, evalCtx, 0)
                        : es3fShaderMatrixTest.getInputValue(es3fShaderMatrixTest.InputType.INPUTTYPE_CONST, in0.dataType, evalCtx, 0);

                    setColor(evalCtx, es3fShaderMatrixTest.reduceToVec3(es3fShaderMatrixTest.transpose(/** @type {tcuMatrix.Matrix} */(in0_))));
                };
        	case es3fShaderMatrixTest.MatrixOp.OP_INVERSE:
                return function (evalCtx) {
                    var in0_ = in0.inputType == es3fShaderMatrixTest.InputType.INPUTTYPE_DYNAMIC ?
                        es3fShaderMatrixTest.getInputValue(in0.inputType, in0.dataType, evalCtx, 0)
                        : es3fShaderMatrixTest.getInputValue(es3fShaderMatrixTest.InputType.INPUTTYPE_CONST, in0.dataType, evalCtx, 0);

                    setColor(evalCtx, es3fShaderMatrixTest.reduceToVec3(es3fShaderMatrixTest.inverse(/** @type {tcuMatrix.Matrix} */(in0_))));
                };
        	case es3fShaderMatrixTest.MatrixOp.OP_DETERMINANT:
                return function (evalCtx) {
                    var in0_ = in0.inputType == es3fShaderMatrixTest.InputType.INPUTTYPE_DYNAMIC ?
                        es3fShaderMatrixTest.getInputValue(in0.inputType, in0.dataType, evalCtx, 0)
                        : es3fShaderMatrixTest.getInputValue(es3fShaderMatrixTest.InputType.INPUTTYPE_CONST, in0.dataType, evalCtx, 0);

                    var det = es3fShaderMatrixTest.determinant(/** @type {tcuMatrix.Matrix} */(in0_));
                    setColor(evalCtx, [det, det, det]);
                };
        	case es3fShaderMatrixTest.MatrixOp.OP_UNARY_PLUS:
                return function (evalCtx) {
                    var in0_ = in0.inputType == es3fShaderMatrixTest.InputType.INPUTTYPE_DYNAMIC ?
                        es3fShaderMatrixTest.getInputValue(in0.inputType, in0.dataType, evalCtx, 0)
                        : es3fShaderMatrixTest.getInputValue(es3fShaderMatrixTest.InputType.INPUTTYPE_CONST, in0.dataType, evalCtx, 0);

                    setColor(evalCtx, es3fShaderMatrixTest.reduceToVec3(/** @type {tcuMatrix.Matrix} */(in0_)));
                };
        	case es3fShaderMatrixTest.MatrixOp.OP_NEGATION:
                return function (evalCtx) {
                    var in0_ = in0.inputType == es3fShaderMatrixTest.InputType.INPUTTYPE_DYNAMIC ?
                        es3fShaderMatrixTest.getInputValue(in0.inputType, in0.dataType, evalCtx, 0)
                        : es3fShaderMatrixTest.getInputValue(es3fShaderMatrixTest.InputType.INPUTTYPE_CONST, in0.dataType, evalCtx, 0);

                    setColor(evalCtx, es3fShaderMatrixTest.reduceToVec3(es3fShaderMatrixTest.negate(/** @type {tcuMatrix.Matrix} */(in0_))));
                };
        	case es3fShaderMatrixTest.MatrixOp.OP_PRE_INCREMENT:
                return function (evalCtx) {
                    var in0_ = in0.inputType == es3fShaderMatrixTest.InputType.INPUTTYPE_DYNAMIC ?
                        es3fShaderMatrixTest.getInputValue(in0.inputType, in0.dataType, evalCtx, 0)
                        : es3fShaderMatrixTest.getInputValue(es3fShaderMatrixTest.InputType.INPUTTYPE_CONST, in0.dataType, evalCtx, 0);

                    var val0 = es3fShaderMatrixTest.reduceToVec3(es3fShaderMatrixTest.increment(/** @type {tcuMatrix.Matrix} */(in0_)));
                    var val1 = es3fShaderMatrixTest.reduceToVec3(es3fShaderMatrixTest.increment(/** @type {tcuMatrix.Matrix} */(in0_)));
                    setColor(evalCtx, deMath.add(val0, val1));
                };
        	case es3fShaderMatrixTest.MatrixOp.OP_PRE_DECREMENT:
                return function (evalCtx) {
                    var in0_ = in0.inputType == es3fShaderMatrixTest.InputType.INPUTTYPE_DYNAMIC ?
                        es3fShaderMatrixTest.getInputValue(in0.inputType, in0.dataType, evalCtx, 0)
                        : es3fShaderMatrixTest.getInputValue(es3fShaderMatrixTest.InputType.INPUTTYPE_CONST, in0.dataType, evalCtx, 0);

                    var val0 = es3fShaderMatrixTest.reduceToVec3(es3fShaderMatrixTest.decrement(/** @type {tcuMatrix.Matrix} */(in0_)));
                    var val1 = es3fShaderMatrixTest.reduceToVec3(es3fShaderMatrixTest.decrement(/** @type {tcuMatrix.Matrix} */(in0_)));
                    setColor(evalCtx, deMath.add(val0, val1));
                };
        	case es3fShaderMatrixTest.MatrixOp.OP_POST_INCREMENT:
                return function (evalCtx) {
                    var in0_ = in0.inputType == es3fShaderMatrixTest.InputType.INPUTTYPE_DYNAMIC ?
                        es3fShaderMatrixTest.getInputValue(in0.inputType, in0.dataType, evalCtx, 0)
                        : es3fShaderMatrixTest.getInputValue(es3fShaderMatrixTest.InputType.INPUTTYPE_CONST, in0.dataType, evalCtx, 0);

                    var val0 = es3fShaderMatrixTest.reduceToVec3((in0_));
                    var val1 = es3fShaderMatrixTest.reduceToVec3(es3fShaderMatrixTest.increment(/** @type {tcuMatrix.Matrix} */(in0_)));
                    setColor(evalCtx, deMath.add(val0, val1));
                };
        	case es3fShaderMatrixTest.MatrixOp.OP_POST_DECREMENT:
                return function (evalCtx) {
                    var in0_ = in0.inputType == es3fShaderMatrixTest.InputType.INPUTTYPE_DYNAMIC ?
                        es3fShaderMatrixTest.getInputValue(in0.inputType, in0.dataType, evalCtx, 0)
                        : es3fShaderMatrixTest.getInputValue(es3fShaderMatrixTest.InputType.INPUTTYPE_CONST, in0.dataType, evalCtx, 0);

                    var val0 = es3fShaderMatrixTest.reduceToVec3((in0_));
                    var val1 = es3fShaderMatrixTest.reduceToVec3(es3fShaderMatrixTest.decrement(/** @type {tcuMatrix.Matrix} */(in0_)));
                    setColor(evalCtx, deMath.add(val0, val1));
                };
        	case es3fShaderMatrixTest.MatrixOp.OP_ADD_INTO:
                return function (evalCtx) {
                    var in0_ = in0.inputType == es3fShaderMatrixTest.InputType.INPUTTYPE_DYNAMIC ?
                        es3fShaderMatrixTest.getInputValue(in0.inputType, in0.dataType, evalCtx, 0)
                        : es3fShaderMatrixTest.getInputValue(es3fShaderMatrixTest.InputType.INPUTTYPE_CONST, in0.dataType, evalCtx, 0);
                    var in1_ = in1.inputType == es3fShaderMatrixTest.InputType.INPUTTYPE_DYNAMIC ?
                        es3fShaderMatrixTest.getInputValue(in1.inputType, in0.dataType, evalCtx, 1)
                        : es3fShaderMatrixTest.getInputValue(es3fShaderMatrixTest.InputType.INPUTTYPE_CONST, in1.dataType, evalCtx, 1);

                    setColor(evalCtx, es3fShaderMatrixTest.reduceToVec3(tcuMatrix.add(/** @type {tcuMatrix.Matrix} */(in0_), /** @type {tcuMatrix.Matrix} */(in1_))));
                };
        	case es3fShaderMatrixTest.MatrixOp.OP_SUBTRACT_FROM:
                return function (evalCtx) {
                    var in0_ = in0.inputType == es3fShaderMatrixTest.InputType.INPUTTYPE_DYNAMIC ?
                        es3fShaderMatrixTest.getInputValue(in0.inputType, in0.dataType, evalCtx, 0)
                        : es3fShaderMatrixTest.getInputValue(es3fShaderMatrixTest.InputType.INPUTTYPE_CONST, in0.dataType, evalCtx, 0);
                    var in1_ = in1.inputType == es3fShaderMatrixTest.InputType.INPUTTYPE_DYNAMIC ?
                        es3fShaderMatrixTest.getInputValue(in1.inputType, in0.dataType, evalCtx, 1)
                        : es3fShaderMatrixTest.getInputValue(es3fShaderMatrixTest.InputType.INPUTTYPE_CONST, in1.dataType, evalCtx, 1);

                    setColor(evalCtx, es3fShaderMatrixTest.reduceToVec3(tcuMatrix.subtract(/** @type {tcuMatrix.Matrix} */(in0_), /** @type {tcuMatrix.Matrix} */(in1_))));
                };
        	case es3fShaderMatrixTest.MatrixOp.OP_MULTIPLY_INTO:
                return function (evalCtx) {
                    var in0_ = in0.inputType == es3fShaderMatrixTest.InputType.INPUTTYPE_DYNAMIC ?
                        es3fShaderMatrixTest.getInputValue(in0.inputType, in0.dataType, evalCtx, 0)
                        : es3fShaderMatrixTest.getInputValue(es3fShaderMatrixTest.InputType.INPUTTYPE_CONST, in0.dataType, evalCtx, 0);
                    var in1_ = in1.inputType == es3fShaderMatrixTest.InputType.INPUTTYPE_DYNAMIC ?
                        es3fShaderMatrixTest.getInputValue(in1.inputType, in0.dataType, evalCtx, 1)
                        : es3fShaderMatrixTest.getInputValue(es3fShaderMatrixTest.InputType.INPUTTYPE_CONST, in1.dataType, evalCtx, 1);

                    setColor(evalCtx, es3fShaderMatrixTest.reduceToVec3(tcuMatrix.multiply(/** @type {tcuMatrix.Matrix} */(in0_), /** @type {tcuMatrix.Matrix} */(in1_))));
                };
        	case es3fShaderMatrixTest.MatrixOp.OP_DIVIDE_INTO:
                return function (evalCtx) {
                    var in0_ = in0.inputType == es3fShaderMatrixTest.InputType.INPUTTYPE_DYNAMIC ?
                        es3fShaderMatrixTest.getInputValue(in0.inputType, in0.dataType, evalCtx, 0)
                        : es3fShaderMatrixTest.getInputValue(es3fShaderMatrixTest.InputType.INPUTTYPE_CONST, in0.dataType, evalCtx, 0);
                    var in1_ = in1.inputType == es3fShaderMatrixTest.InputType.INPUTTYPE_DYNAMIC ?
                        es3fShaderMatrixTest.getInputValue(in1.inputType, in1.dataType, evalCtx, 1)
                        : es3fShaderMatrixTest.getInputValue(es3fShaderMatrixTest.InputType.INPUTTYPE_CONST, in1.dataType, evalCtx, 1);

                    setColor(evalCtx, es3fShaderMatrixTest.reduceToVec3(tcuMatrix.divide(/** @type {tcuMatrix.Matrix} */(in0_), /** @type {tcuMatrix.Matrix} */(in1_))));
                };
        }
	};

    /**
     * @constructor
     * @param {es3fShaderMatrixTest.MatrixShaderEvalFunc} evalFunc
     * @param {es3fShaderMatrixTest.InputType} inType0
     * @param {es3fShaderMatrixTest.InputType} inType1
     * @extends {glsShaderRenderCase.ShaderEvaluator}
     */
    es3fShaderMatrixTest.MatrixShaderEvaluator = function(evalFunc, inType0, inType1) {
        glsShaderRenderCase.ShaderEvaluator.call(this);
        this.m_matEvalFunc = evalFunc;
        this.m_inType0 = inType0;
        this.m_inType1 = inType1;
    };

    es3fShaderMatrixTest.MatrixShaderEvaluator.prototype = Object.create(glsShaderRenderCase.ShaderEvaluator);
    es3fShaderMatrixTest.MatrixShaderEvaluator.prototype.constructor = es3fShaderMatrixTest.MatrixShaderEvaluator;

    /**
     * @param {glsShaderRenderCase.ShaderEvalContext} evalCtx
     */
    es3fShaderMatrixTest.MatrixShaderEvaluator.prototype.evaluate = function (evalCtx) {
    	this.m_matEvalFunc(evalCtx);
    }

    /**
     * @param {Array<number>} v
     * @param {number} size
     */
    es3fShaderMatrixTest.writeVectorConstructor = function (v, size) {
    	var str = 'vec' + size + '';
    	for (var ndx = 0; ndx < size; ndx++) {
    		if (ndx != 0)
    			str += ', ';
    		str += v[ndx].toString;
    	}
    	str += ')';
        return str;
    }

    /**
     * @param {tcuMatrix.Matrix} m
     */
    es3fShaderMatrixTest.writeMatrixConstructor = function (m) {
        var str = '';
        if (m.rows == m.cols)
    		str += 'mat' + m.cols;
    	else
    		str += 'mat' + m.cols + 'x' + m.rows;

    	str += '(';
    	for (var colNdx = 0; colNdx < m.cols; colNdx++) {
    		for (var rowNdx = 0; rowNdx < m.rows; rowNdx++) {
    			if (rowNdx > 0 || colNdx > 0)
    				str += ', ';
    			str += m.get(rowNdx, colNdx).toString();
    		}
    	}
    	str += ')';
        return str;
    };

    /**
     * @constructor
     * @param {string} name
     * @param {string} desc
     * @param {es3fShaderMatrixTest.ShaderInput} in0
     * @param {es3fShaderMatrixTest.ShaderInput} in1
     * @param {es3fShaderMatrixTest.MatrixOp} op
     * @param {boolean} isVertexCase
     * @extends {glsShaderRenderCase.ShaderRenderCase}
     */
    es3fShaderMatrixTest.ShaderMatrixCase = function(name, desc, in0, in1, op, isVertexCase) {
        var evalFunc = es3fShaderMatrixTest.getEvalFunc(in0, in1, op);
        glsShaderRenderCase.ShaderRenderCase.call(this, name, desc, isVertexCase, evalFunc);
        this.m_in0 = in0;
        this.m_in1 = in1;
        this.m_op = op;
        this.m_evaluator = new es3fShaderMatrixTest.MatrixShaderEvaluator(evalFunc, in0.inputType, in1.inputType);
    };

    es3fShaderMatrixTest.ShaderMatrixCase.prototype = Object.create(glsShaderRenderCase.ShaderRenderCase.prototype);
    es3fShaderMatrixTest.ShaderMatrixCase.prototype.constructor = es3fShaderMatrixTest.ShaderMatrixCase;


    es3fShaderMatrixTest.ShaderMatrixCase.prototype.init = function () {
        var shaderSources = [ '', '' ];
        var vtx = 0;
    	var frag = 1;
    	var op = this.m_isVertexCase ? vtx : frag;

        /** @type {boolean} */ var isInDynMat0 = gluShaderUtil.isDataTypeMatrix(this.m_in0.dataType) && this.m_in0.inputType == es3fShaderMatrixTest.InputType.INPUTTYPE_DYNAMIC;
    	/** @type {boolean} */ var isInDynMat1 = gluShaderUtil.isDataTypeMatrix(this.m_in1.dataType) && this.m_in1.inputType == es3fShaderMatrixTest.InputType.INPUTTYPE_DYNAMIC;
    	/** @type {Array<string>} */ var inValues = [];
    	/** @type {gluShaderUtil.DataType} */ var resultType;
    	/** @type {gluShaderUtil.precision} */ var resultPrec = this.m_in0.precision;
    	/** @type {Array<string>} */ var passVars = [];
    	/** @type {number} */ var numInputs = (es3fShaderMatrixTest.isOperationBinary(this.m_op)) ? (2) : (1);

    	/** @type {string} */ var operationValue0 = '';
    	/** @type {string} */ var operationValue1 = '';

        if (isInDynMat0 && isInDynMat1) {
            throw new Error ('Only single dynamic matrix input is allowed.');
        }

        if (this.m_op == es3fShaderMatrixTest.MatrixOp.OP_MUL && gluShaderUtil.isDataTypeMatrix(this.m_in0.dataType) && gluShaderUtil.isDataTypeMatrix(this.m_in1.dataType)) {
    		resultType = gluShaderUtil.getDataTypeMatrix(gluShaderUtil.getDataTypeMatrixNumColumns(this.m_in1.dataType), gluShaderUtil.getDataTypeMatrixNumRows(this.m_in0.dataType));
    	} else if (this.m_op == es3fShaderMatrixTest.MatrixOp.OP_OUTER_PRODUCT) {
    		resultType = gluShaderUtil.getDataTypeMatrix(gluShaderUtil.getDataTypeScalarSize(this.m_in1.dataType), gluShaderUtil.getDataTypeScalarSize(this.m_in0.dataType));
    	} else if (this.m_op == es3fShaderMatrixTest.MatrixOp.OP_TRANSPOSE) {
    		resultType = gluShaderUtil.getDataTypeMatrix(gluShaderUtil.getDataTypeMatrixNumRows(this.m_in0.dataType), gluShaderUtil.getDataTypeMatrixNumColumns(this.m_in0.dataType));
    	} else if (this.m_op == es3fShaderMatrixTest.MatrixOp.OP_INVERSE) {
    		resultType = this.m_in0.dataType;
    	} else if (this.m_op == es3fShaderMatrixTest.MatrixOp.OP_DETERMINANT) {
    		resultType = gluShaderUtil.DataType.FLOAT;
    	} else if (es3fShaderMatrixTest.getOperationType(this.m_op) == es3fShaderMatrixTest.OperationType.OPERATIONTYPE_UNARY_PREFIX_OPERATOR ||
    			 es3fShaderMatrixTest.getOperationType(this.m_op) == es3fShaderMatrixTest.OperationType.OPERATIONTYPE_UNARY_POSTFIX_OPERATOR) {
    		resultType = this.m_in0.dataType;
    	} else if (gluShaderUtil.isDataTypeMatrix(this.m_in0.dataType) && gluShaderUtil.isDataTypeMatrix(this.m_in1.dataType)) {
    		if (this.m_in0.dataType !== this.m_in1.dataType) {
                throw new Error ('Incompatible data types');
            }
    		resultType = this.m_in0.dataType;
    	} else if (gluShaderUtil.isDataTypeMatrix(this.m_in0.dataType) || gluShaderUtil.isDataTypeMatrix(this.m_in1.dataType)) {
    		/** @type {number} */ var matNdx = gluShaderUtil.isDataTypeMatrix(this.m_in0.dataType) ? 0 : 1;
    		/** @type {gluShaderUtil.DataType} */ var matrixType = matNdx == 0 ? this.m_in0.dataType : this.m_in1.dataType;
    		/** @type {gluShaderUtil.DataType} */ var otherType = matNdx == 0 ? this.m_in1.dataType : this.m_in0.dataType;

    		if (otherType == gluShaderUtil.DataType.FLOAT)
    			resultType = matrixType;
    		else  {
    			if (!gluShaderUtil.isDataTypeVector(otherType)) {
                    throw new Error ('Is not data type vector');
                }
    			resultType = gluShaderUtil.getDataTypeFloatVec(matNdx == 0 ? gluShaderUtil.getDataTypeMatrixNumRows(matrixType) : gluShaderUtil.getDataTypeMatrixNumColumns(matrixType));
    		}
    	} else {
    		throw new Error ('Error');
    	}

        shaderSources[vtx] += '#version 300 es\n';
    	shaderSources[frag] += '#version 300 es\n';

    	shaderSources[vtx] += 'in highp vec4 a_position;\n';
    	shaderSources[frag] += 'layout(location = 0) out mediump vec4 dEQP_FragColor;\n';
    	if (this.m_isVertexCase) {
    		shaderSources[vtx] += 'out mediump vec4 v_color;\n';
    		shaderSources[frag] += 'in mediump vec4 v_color;\n';
    	}

        // Input declarations.
    	for (var inNdx = 0; inNdx < numInputs; inNdx++) {
    		/** @type {es3fShaderMatrixTest.ShaderInput} */ var ind = inNdx > 0 ? this.m_in1 : this.m_in0;
    		/** @type {string} */ var precName = gluShaderUtil.getPrecisionName(ind.precision);
    		/** @type {string} */ var typeName = gluShaderUtil.getDataTypeName(ind.dataType);
    		/** @type {number} */ var inValueNdx = inNdx > 0 ? 1 : 0;

    		if (ind.inputType == es3fShaderMatrixTest.InputType.INPUTTYPE_DYNAMIC) {
    			shaderSources[vtx] += 'in ' + precName + ' ' + typeName + ' a_';

    			if (gluShaderUtil.isDataTypeMatrix(ind.dataType)) {
    				// a_matN, v_matN
    				shaderSources[vtx] += typeName + ';\n';
    				if (!this.m_isVertexCase) {
    					shaderSources[vtx] += 'out ' + precName + ' ' + typeName + ' v_' + typeName + ';\n';
    					shaderSources[frag] += 'in ' + precName + ' ' + typeName + ' v_' + typeName + ';\n';
    					passVars.push(typeName);
    				}

    				inValues[inValueNdx] = (this.m_isVertexCase ? 'a_' : 'v_') + gluShaderUtil.getDataTypeName(ind.dataType);
    			} else {
    				// a_coords, v_coords
    				shaderSources[vtx] += 'coords;\n';
    				if (!this.m_isVertexCase) {
    					shaderSources[vtx] += 'out ' + precName + ' ' + typeName + ' v_coords;\n';
    					shaderSources[frag] += 'in ' + precName + ' ' + typeName + ' v_coords;\n';
    					passVars.push('coords');
    				}

    				inValues[inValueNdx] = this.m_isVertexCase ? 'a_coords' : 'v_coords';
    			}
    		}  else if (ind.inputType == es3fShaderMatrixTest.InputType.INPUTTYPE_UNIFORM) {
    			shaderSources[op] += 'uniform ' + precName + ' ' + typeName + ' u_in' + inNdx + ';\n';
    			inValues[inValueNdx] = 'u_in' + inNdx.toString();
    		} else if (ind.inputType == es3fShaderMatrixTest.InputType.INPUTTYPE_CONST) {
    			shaderSources[op] += 'const ' + precName + ' ' + typeName + ' in' + inNdx + ' = ';

    			// Generate declaration.
    			switch (ind.dataType) {
    				case gluShaderUtil.DataType.FLOAT:
                        shaderSources[op] += s_constInFloat[inNdx].toString();
                        break;
    				case gluShaderUtil.DataType.FLOAT_VEC2:
                        shaderSources[op] += es3fShaderMatrixTest.writeVectorConstructor( s_constInVec2[inNdx], 2);
                        break;
    				case gluShaderUtil.DataType.FLOAT_VEC3:
                        shaderSources[op] += es3fShaderMatrixTest.writeVectorConstructor( s_constInVec3[inNdx], 3);
                        break;
    				case gluShaderUtil.DataType.FLOAT_VEC4:
                        shaderSources[op] += es3fShaderMatrixTest.writeVectorConstructor( s_constInVec4[inNdx], 4);
                        break;
    				case gluShaderUtil.DataType.FLOAT_MAT2:
                        shaderSources[op] += es3fShaderMatrixTest.writeMatrixConstructor( tcuMatrix.matrixFromDataArray(2, 2, s_constInMat2x2[inNdx]));
                        break;
    				case gluShaderUtil.DataType.FLOAT_MAT2X3:
                        shaderSources[op] += es3fShaderMatrixTest.writeMatrixConstructor( tcuMatrix.matrixFromDataArray(3, 2, s_constInMat2x3[inNdx]));
                        break;
    				case gluShaderUtil.DataType.FLOAT_MAT2X4:
                        shaderSources[op] += es3fShaderMatrixTest.writeMatrixConstructor( tcuMatrix.matrixFromDataArray(4, 2, s_constInMat2x4[inNdx]));
                        break;
    				case gluShaderUtil.DataType.FLOAT_MAT3X2:
                        shaderSources[op] += es3fShaderMatrixTest.writeMatrixConstructor( tcuMatrix.matrixFromDataArray(2, 3, s_constInMat3x2[inNdx]));
                        break;
    				case gluShaderUtil.DataType.FLOAT_MAT3:
                        shaderSources[op] += es3fShaderMatrixTest.writeMatrixConstructor( tcuMatrix.matrixFromDataArray(3, 3, s_constInMat3x3[inNdx]));
                        break;
    				case gluShaderUtil.DataType.FLOAT_MAT3X4:
                        shaderSources[op] += es3fShaderMatrixTest.writeMatrixConstructor( tcuMatrix.matrixFromDataArray(4, 3, s_constInMat3x4[inNdx]));
                        break;
    				case gluShaderUtil.DataType.FLOAT_MAT4X2:
                        shaderSources[op] += es3fShaderMatrixTest.writeMatrixConstructor( tcuMatrix.matrixFromDataArray(2, 4, s_constInMat4x2[inNdx]));
                        break;
    				case gluShaderUtil.DataType.FLOAT_MAT4X3:
                        shaderSources[op] += es3fShaderMatrixTest.writeMatrixConstructor( tcuMatrix.matrixFromDataArray(3, 4, s_constInMat4x3[inNdx]));
                        break;
    				case gluShaderUtil.DataType.FLOAT_MAT4:
                        shaderSources[op] += es3fShaderMatrixTest.writeMatrixConstructor( tcuMatrix.matrixFromDataArray(4, 4, s_constInMat4x4[inNdx]));
                        break;

    				default:
    					throw new Error('Data type error');
    			}

    			shaderSources[op] += ';\n';

    			inValues[inValueNdx] = 'in' + inNdx.toString();
    		}
        }

        shaderSources[vtx] += '\n'
		+ 'void main (void)\n'
		+ '{\n'
		+ '	gl_Position = a_position;\n';
        shaderSources[frag] += '\n'
        + 'void main (void)\n'
        + '{\n';

    	if (this.m_isVertexCase)
    		shaderSources[frag] += '	dEQP_FragColor = v_color;\n';
    	else {
    		for (var i = 0; i != passVars.length; i++)
    			shaderSources[vtx] += '	v_' + passVars[i] + ' = ' + 'a_' + passVars[i] + ';\n';
    	}

    	// Operation.

    	switch (es3fShaderMatrixTest.getOperationNature(this.m_op)) {
    		case es3fShaderMatrixTest.OperationNature.OPERATIONNATURE_PURE:
    			if (es3fShaderMatrixTest.getOperationType(this.m_op) == es3fShaderMatrixTest.OperationType.OPERATIONTYPE_ASSIGNMENT)
                    throw new Error('Wrong operation type');

    			operationValue0 = inValues[0];
    			operationValue1 = inValues[1];
    			break;

    		case es3fShaderMatrixTest.OperationNature.OPERATIONNATURE_MUTATING:
    			if (es3fShaderMatrixTest.getOperationType(this.m_op) == es3fShaderMatrixTest.OperationType.OPERATIONTYPE_ASSIGNMENT)
                    throw new Error('Wrong operation type');

    			shaderSources[op] += '	' + gluShaderUtil.getPrecisionName(resultPrec) + ' ' + gluShaderUtil.getDataTypeName(resultType) + ' tmpValue = ' + inValues[0] + ';\n';

    			operationValue0 = 'tmpValue';
    			operationValue1 = inValues[1];
    			break;

    		case es3fShaderMatrixTest.OperationNature.OPERATIONNATURE_ASSIGNMENT:
    			if (es3fShaderMatrixTest.getOperationType(this.m_op) != es3fShaderMatrixTest.OperationType.OPERATIONTYPE_ASSIGNMENT)
                    throw new Error('Wrong operation type');

    			operationValue0 = inValues[0];
    			operationValue1 = inValues[1];
    			break;

    		default:
    		    throw new Error('Wrong operation nature');
    	}

        switch (es3fShaderMatrixTest.getOperationType(this.m_op)) {
    		case es3fShaderMatrixTest.OperationType.OPERATIONTYPE_BINARY_OPERATOR:
    			shaderSources[op] += '	' + gluShaderUtil.getPrecisionName(resultPrec) + ' '
                + gluShaderUtil.getDataTypeName(resultType)
                + ' res = ' + operationValue0 + ' '
                + es3fShaderMatrixTest.getOperationName(this.m_op) + ' '
                + operationValue1 + ';\n';
    			break;
    		case es3fShaderMatrixTest.OperationType.OPERATIONTYPE_UNARY_PREFIX_OPERATOR:
    			shaderSources[op] += '	' + gluShaderUtil.getPrecisionName(resultPrec) + ' '
                + gluShaderUtil.getDataTypeName(resultType)
                + ' res = ' + es3fShaderMatrixTest.getOperationName(this.m_op)
                + operationValue0 + ';\n';
    			break;
    		case es3fShaderMatrixTest.OperationType.OPERATIONTYPE_UNARY_POSTFIX_OPERATOR:
    			shaderSources[op] += '	' + gluShaderUtil.getPrecisionName(resultPrec) + ' '
                + gluShaderUtil.getDataTypeName(resultType)
                + ' res = ' + operationValue0
                + es3fShaderMatrixTest.getOperationName(this.m_op) + ';\n';
    			break;
    		case es3fShaderMatrixTest.OperationType.OPERATIONTYPE_BINARY_FUNCTION:
    			shaderSources[op] += '	' + gluShaderUtil.getPrecisionName(resultPrec)
                + ' ' + gluShaderUtil.getDataTypeName(resultType)
                + ' res = ' + es3fShaderMatrixTest.getOperationName(this.m_op)
                + '(' + operationValue0
                + ', ' + operationValue1 + ');\n';
    			break;
    		case es3fShaderMatrixTest.OperationType.OPERATIONTYPE_UNARY_FUNCTION:
    			shaderSources[op] += '	' + gluShaderUtil.getPrecisionName(resultPrec)
                + ' ' + gluShaderUtil.getDataTypeName(resultType)
                + ' res = ' + es3fShaderMatrixTest.getOperationName(this.m_op)
                + '(' + operationValue0 + ');\n';
    			break;
    		case es3fShaderMatrixTest.OperationType.OPERATIONTYPE_ASSIGNMENT:
    			shaderSources[op] += '	' + gluShaderUtil.getPrecisionName(resultPrec)
                + ' ' + gluShaderUtil.getDataTypeName(resultType)
                + ' res = ' + operationValue0 + ';\n';
    			shaderSources[op] += '	res ' + es3fShaderMatrixTest.getOperationName(this.m_op)
                + ' ' + operationValue1 + ';\n';
    			break;
    		default:
    			throw new Error('Wrong operation type');
    	}

        // Reduction to vec3 (rgb). Check the used value too if it was modified
    	shaderSources[op] +=  '	' + (this.m_isVertexCase ? 'v_color' : 'dEQP_FragColor') + ' = ';

    	if (es3fShaderMatrixTest.isOperationValueModifying(this.m_op))
    		shaderSources[op] +=  'vec4(' + this.genGLSLMatToVec3Reduction(resultType, 'res')
            + ', 1.0) + vec4(' + this.genGLSLMatToVec3Reduction(resultType, 'tmpValue')
            + ', 0.0);\n';
    	else
    		shaderSources[op] +=  'vec4(' + this.genGLSLMatToVec3Reduction(resultType, 'res')
            + ', 1.0);\n';

    	shaderSources[vtx] += '}\n';
    	shaderSources[frag] += '}\n';

    	this.m_vertShaderSource	= shaderSources[vtx];
    	this.m_fragShaderSource	= shaderSources[frag];

        // \todo [2012-02-14 pyry] Compute better values for matrix tests.
    	for (var attribNdx = 0; attribNdx < 4; attribNdx++) {
    		this.m_userAttribTransforms[attribNdx] = new tcuMatrix.Matrix(4, 4, 0);
    		this.m_userAttribTransforms[attribNdx].set(0, 3, 0.2);// !< prevent matrix*vec from going into zero (assuming vec.w != 0)
    		this.m_userAttribTransforms[attribNdx].set(1, 3, 0.1);// !<
    		this.m_userAttribTransforms[attribNdx].set(2, 3, 0.4 + 0.15 * attribNdx);// !<
    		this.m_userAttribTransforms[attribNdx].set(3, 3, 0.7);// !<
    		this.m_userAttribTransforms[attribNdx].set((0 + attribNdx) % 4, 0, 1.0);
    		this.m_userAttribTransforms[attribNdx].set((1 + attribNdx) % 4, 1, 1.0);
    		this.m_userAttribTransforms[attribNdx].set((2 + attribNdx) % 4, 2, 1.0);
    		this.m_userAttribTransforms[attribNdx].set((3 + attribNdx) % 4, 3, 1.0);
    	}

    	// prevent bad reference cases such as black result images by fine-tuning used matrices
    	if (es3fShaderMatrixTest.getOperationTestMatrixType(this.m_op) != es3fShaderMatrixTest.MatrixType.TESTMATRIXTYPE_DEFAULT) {
    		for (var attribNdx = 0; attribNdx < 4; attribNdx++) {
    			for (var row = 0; row < 4; row++)
        			for (var col = 0; col < 4; col++) {
        				switch (es3fShaderMatrixTest.getOperationTestMatrixType(this.m_op)) {
        					case es3fShaderMatrixTest.MatrixType.TESTMATRIXTYPE_NEGATED:
        						this.m_userAttribTransforms[attribNdx].set(row, col, -this.m_userAttribTransforms[attribNdx].get(row, col));
        						break;
        					case es3fShaderMatrixTest.MatrixType.TESTMATRIXTYPE_INCREMENTED:
        						this.m_userAttribTransforms[attribNdx].set(row, col, this.m_userAttribTransforms[attribNdx].get(row, col) + 0.3);
        						break;
        					case es3fShaderMatrixTest.MatrixType.TESTMATRIXTYPE_DECREMENTED:
        						this.m_userAttribTransforms[attribNdx].set(row, col, this.m_userAttribTransforms[attribNdx].get(row, col) - 0.3);
        						break;
        					case es3fShaderMatrixTest.MatrixType.TESTMATRIXTYPE_NEGATED_INCREMENTED:
        						this.m_userAttribTransforms[attribNdx].set(row, col,  -(this.m_userAttribTransforms[attribNdx].get(row, col) + 0.3));
        						break;
        					case es3fShaderMatrixTest.MatrixType.TESTMATRIXTYPE_INCREMENTED_LESS:
        						this.m_userAttribTransforms[attribNdx].set(row, col, this.m_userAttribTransforms[attribNdx].get(row, col) - 0.1);
        						break;
        					default:
        						throw new Error('Wrong Matrix type');
        				}
        			}
    		}
    	}

        glsShaderRenderCase.ShaderRenderCase.prototype.init.call(this);
    };


    es3fShaderMatrixTest.ShaderMatrixCase.prototype.setupUniforms = function(programId, constCoords) {
        for (var inNdx = 0; inNdx < 2; inNdx++)
        {
            var input = inNdx > 0 ? this.m_in1 : this.m_in0;

            if (input.inputType == es3fShaderMatrixTest.InputType.INPUTTYPE_UNIFORM)
            {
                var loc = gl.getUniformLocation(programId, "u_in" + inNdx);

                if (!loc)
                    continue;

                switch (input.dataType)
                {
                    case gluShaderUtil.DataType.FLOAT:        gl.uniform1f(loc, s_constInFloat[inNdx]);                       break;
                    case gluShaderUtil.DataType.FLOAT_VEC2:   gl.uniform2fv(loc, s_constInVec2[inNdx]);           break;
                    case gluShaderUtil.DataType.FLOAT_VEC3:   gl.uniform3fv(loc, s_constInVec3[inNdx]);           break;
                    case gluShaderUtil.DataType.FLOAT_VEC4:   gl.uniform4fv(loc, s_constInVec4[inNdx]);           break;
                    // \note GLES3 supports transpose in matrix upload.
                    case gluShaderUtil.DataType.FLOAT_MAT2:   gl.uniformMatrix2fv (loc, true, s_constInMat2x2[inNdx]);  break;
                    case gluShaderUtil.DataType.FLOAT_MAT2X3: gl.uniformMatrix2x3fv(loc, true, s_constInMat2x3[inNdx]); break;
                    case gluShaderUtil.DataType.FLOAT_MAT2X4: gl.uniformMatrix2x4fv(loc, true, s_constInMat2x4[inNdx]); break;
                    case gluShaderUtil.DataType.FLOAT_MAT3X2: gl.uniformMatrix3x2fv(loc, true, s_constInMat3x2[inNdx]); break;
                    case gluShaderUtil.DataType.FLOAT_MAT3:   gl.uniformMatrix3fv (loc, true, s_constInMat3x3[inNdx]);  break;
                    case gluShaderUtil.DataType.FLOAT_MAT3X4: gl.uniformMatrix3x4fv(loc, true, s_constInMat3x4[inNdx]); break;
                    case gluShaderUtil.DataType.FLOAT_MAT4X2: gl.uniformMatrix4x2fv(loc, true, s_constInMat4x2[inNdx]); break;
                    case gluShaderUtil.DataType.FLOAT_MAT4X3: gl.uniformMatrix4x3fv(loc, true, s_constInMat4x3[inNdx]); break;
                    case gluShaderUtil.DataType.FLOAT_MAT4:   gl.uniformMatrix4fv (loc, true, s_constInMat4x4[inNdx]);  break;
                    default:
                        throw new Error('Invalid datatype' + input.dataType);
                }
            }
        }
    };



    /**
     * @param {gluShaderUtil.DataType} matType
     * @param {string} varName
     * @return {string}
     */
    es3fShaderMatrixTest.ShaderMatrixCase.prototype.genGLSLMatToVec3Reduction = function (matType, varName) {
    	/** @type {string} */ var op = '';

    	switch (matType) {
    		case gluShaderUtil.DataType.FLOAT:
                op += varName + ', '
                + varName + ', '
                + varName + '';
                break;
    		case gluShaderUtil.DataType.FLOAT_VEC2:
                op += varName + '.x, '
                + varName + '.y, '
                + varName + '.x';
                break;
    		case gluShaderUtil.DataType.FLOAT_VEC3:
                op += varName + '';
                break;
    		case gluShaderUtil.DataType.FLOAT_VEC4:
                op += varName + '.x, '
                + varName + '.y, '
                + varName + '.z+'
                + varName + '.w';
                break;
    		case gluShaderUtil.DataType.FLOAT_MAT2:
                op += varName + '[0][0], '
                + varName + '[1][0], '
                + varName + '[0][1]+'
                + varName + '[1][1]';
                break;
    		case gluShaderUtil.DataType.FLOAT_MAT2X3:
                op += varName + '[0] + '
                + varName + '[1]';
                break;
    		case gluShaderUtil.DataType.FLOAT_MAT2X4:
                op += varName + '[0].xyz + '
                + varName + '[1].yzw';
                break;
    		case gluShaderUtil.DataType.FLOAT_MAT3X2:
                op += varName + '[0][0]+'
                + varName + '[0][1], '
                + varName + '[1][0]+'
                + varName + '[1][1], '
                + varName + '[2][0]+'
                + varName + '[2][1]';
                break;
    		case gluShaderUtil.DataType.FLOAT_MAT3:
                op += varName + '[0] + '
                + varName + '[1] + '
                + varName + '[2]';
                break;
    		case gluShaderUtil.DataType.FLOAT_MAT3X4:
                op += varName + '[0].xyz + '
                + varName + '[1].yzw + '
                + varName + '[2].zwx';
                break;
    		case gluShaderUtil.DataType.FLOAT_MAT4X2:
                op += varName + '[0][0]+'
                + varName + '[0][1]+'
                + varName + '[3][0], '
                + varName + '[1][0]+'
                + varName + '[1][1]+'
                + varName + '[3][1], '
                + varName + '[2][0]+'
                + varName + '[2][1]';
                break;
    		case gluShaderUtil.DataType.FLOAT_MAT4X3:
                op += varName + '[0] + '
                + varName + '[1] + '
                + varName + '[2] + '
                + varName + '[3]';
                break;
    		case gluShaderUtil.DataType.FLOAT_MAT4:
                op += varName + '[0].xyz+'
                + varName + '[1].yzw+'
                + varName + '[2].zwx+'
                + varName + '[3].wxy';
                break;

    		default:
    			throw new Error('Wrong data type');
    	}

    	return op;
    }

    /**
     * @constructor
     * @param {string} name
     * @param {string} desc
     * @param {es3fShaderMatrixTest.MatrixOp} op
     * @param {boolean} extendedInputTypeCases
     * @param {boolean} createInputTypeGroup
     */
    es3fShaderMatrixTest.ops = function (name, desc, op, extendedInputTypeCases, createInputTypeGroup) {
        this.name = name;
        this.desc = desc;
        this.op = op;
        this.extendedInputTypeCases = extendedInputTypeCases;
        this.createInputTypeGroup = createInputTypeGroup;
    };

    /**
     * @constructor
     * @param {string} name
     * @param {string} desc
     * @param {es3fShaderMatrixTest.InputType} type
     */
	es3fShaderMatrixTest.InputTypeSpec = function (name, desc, type) {
		this.name = name;
		this.desc = desc;
		this.type = type;
	};

    es3fShaderMatrixTest.init = function () {
        var state = tcuTestCase.runner;

        var ops = [
            new es3fShaderMatrixTest.ops('add', 'Matrix addition tests', es3fShaderMatrixTest.MatrixOp.OP_ADD, true, true),
            new es3fShaderMatrixTest.ops('sub', 'Matrix subtraction tests', es3fShaderMatrixTest.MatrixOp.OP_SUB, true, true),
            new es3fShaderMatrixTest.ops('mul', 'Matrix multiplication tests', es3fShaderMatrixTest.MatrixOp.OP_MUL, true,	true),
            new es3fShaderMatrixTest.ops('div', 'Matrix division tests', es3fShaderMatrixTest.MatrixOp.OP_DIV, true, true),
            new es3fShaderMatrixTest.ops('matrixcompmult', 'Matrix component-wise multiplication tests', es3fShaderMatrixTest.MatrixOp.OP_COMP_MUL, false, true),
            new es3fShaderMatrixTest.ops('outerproduct', 'Matrix outerProduct() tests', es3fShaderMatrixTest.MatrixOp.OP_OUTER_PRODUCT, false, true),
            new es3fShaderMatrixTest.ops('transpose', 'Matrix transpose() tests', es3fShaderMatrixTest.MatrixOp.OP_TRANSPOSE, false, true),
            new es3fShaderMatrixTest.ops('determinant', 'Matrix determinant() tests', es3fShaderMatrixTest.MatrixOp.OP_DETERMINANT, false, true),
            new es3fShaderMatrixTest.ops('inverse', 'Matrix inverse() tests', es3fShaderMatrixTest.MatrixOp.OP_INVERSE, false, true),
            new es3fShaderMatrixTest.ops('unary_addition', 'Matrix unary addition tests', es3fShaderMatrixTest.MatrixOp.OP_UNARY_PLUS, false, false),
            new es3fShaderMatrixTest.ops('negation', 'Matrix negation tests', es3fShaderMatrixTest.MatrixOp.OP_NEGATION, false, false),
            new es3fShaderMatrixTest.ops('pre_increment', 'Matrix prefix increment tests', es3fShaderMatrixTest.MatrixOp.OP_PRE_INCREMENT, false, false),
            new es3fShaderMatrixTest.ops('pre_decrement', 'Matrix prefix decrement tests', es3fShaderMatrixTest.MatrixOp.OP_PRE_DECREMENT, false, false),
            new es3fShaderMatrixTest.ops('post_increment', 'Matrix postfix increment tests', es3fShaderMatrixTest.MatrixOp.OP_POST_INCREMENT, false, false),
            new es3fShaderMatrixTest.ops('post_decrement', 'Matrix postfix decrement tests', es3fShaderMatrixTest.MatrixOp.OP_POST_DECREMENT, false, false),
            new es3fShaderMatrixTest.ops('add_assign', 'Matrix add into tests', es3fShaderMatrixTest.MatrixOp.OP_ADD_INTO, false, false),
            new es3fShaderMatrixTest.ops('sub_assign', 'Matrix subtract from tests', es3fShaderMatrixTest.MatrixOp.OP_SUBTRACT_FROM,false, false),
            new es3fShaderMatrixTest.ops('mul_assign', 'Matrix multiply into tests', es3fShaderMatrixTest.MatrixOp.OP_MULTIPLY_INTO,false, false),
            new es3fShaderMatrixTest.ops('div_assign', 'Matrix divide into tests', es3fShaderMatrixTest.MatrixOp.OP_DIVIDE_INTO,false, false)
    	];

    	var extendedInputTypes = [
            new es3fShaderMatrixTest.InputTypeSpec('const', 'Constant matrix input', es3fShaderMatrixTest.InputType.INPUTTYPE_CONST),
            new es3fShaderMatrixTest.InputTypeSpec('uniform', 'Uniform matrix input', es3fShaderMatrixTest.InputType.INPUTTYPE_UNIFORM),
            new es3fShaderMatrixTest.InputTypeSpec('dynamic', 'Dynamic matrix input', es3fShaderMatrixTest.InputType.INPUTTYPE_DYNAMIC)
    	];

    	var reducedInputTypes = [
            new es3fShaderMatrixTest.InputTypeSpec('dynamic', 'Dynamic matrix input', es3fShaderMatrixTest.InputType.INPUTTYPE_DYNAMIC)
    	];

    	/** @type {Array<gluShaderUtil.DataType>} */ var matrixTypes = [
            gluShaderUtil.DataType.FLOAT_MAT2,
            gluShaderUtil.DataType.FLOAT_MAT2X3,
            gluShaderUtil.DataType.FLOAT_MAT2X4,
            gluShaderUtil.DataType.FLOAT_MAT3X2,
            gluShaderUtil.DataType.FLOAT_MAT3,
            gluShaderUtil.DataType.FLOAT_MAT3X4,
            gluShaderUtil.DataType.FLOAT_MAT4X2,
            gluShaderUtil.DataType.FLOAT_MAT4X3,
            gluShaderUtil.DataType.FLOAT_MAT4
    	];

    	/** @type {Array<gluShaderUtil.precision>} */ var precisions = [
            gluShaderUtil.precision.PRECISION_LOWP,
            gluShaderUtil.precision.PRECISION_MEDIUMP,
            gluShaderUtil.precision.PRECISION_HIGHP
    	];

    	for (var opNdx = 0; opNdx < ops.length; opNdx++) {
            var inTypeList = ops[opNdx].extendedInputTypeCases ? extendedInputTypes : reducedInputTypes;
            var inTypeListSize = ops[opNdx].extendedInputTypeCases ? extendedInputTypes.length : reducedInputTypes.length;
            var op = ops[opNdx].op;

            for (var inTypeNdx = 0; inTypeNdx < inTypeListSize; inTypeNdx++) {
                var inputType = inTypeList[inTypeNdx].type;
                var group = [];

                if (ops[opNdx].name != 'mul') {
                    if (ops[opNdx].createInputTypeGroup) {
                        group[0] = tcuTestCase.newTest(ops[opNdx].name + '.' + inTypeList[inTypeNdx].name, inTypeList[inTypeNdx].desc);
                    } else {
                        group[0] = tcuTestCase.newTest(ops[opNdx].name, ops[opNdx].desc);
                    }
                    state.testCases.addChild(group[0]);
                } else {
                    for (var ii = 0; ii < precisions.length; ++ii) {
                        group[ii] = tcuTestCase.newTest(ops[opNdx].name + '.' + inTypeList[inTypeNdx].name, inTypeList[inTypeNdx].desc);
                        state.testCases.addChild(group[ii]);
                    }
                }

                for (var matTypeNdx = 0; matTypeNdx < matrixTypes.length; matTypeNdx++) {
                    var matType = matrixTypes[matTypeNdx];
                    var numCols = gluShaderUtil.getDataTypeMatrixNumColumns(matType);
                    var numRows = gluShaderUtil.getDataTypeMatrixNumRows(matType);
                    var matTypeName = gluShaderUtil.getDataTypeName(matType);

                    for (var precNdx = 0; precNdx < precisions.length; precNdx++) {
                        var inGroup;
                        if (ops[opNdx].name != 'mul') {
                            inGroup = group[0];
                        } else {
                            inGroup = group[precNdx];
                        }

                        var precision = precisions[precNdx];
                        var precName = gluShaderUtil.getPrecisionName(precision);
                        var baseName = precName + '_' + matTypeName + '_';
                        var matIn = new es3fShaderMatrixTest.ShaderInput(inputType, matType, precision);

                        if (es3fShaderMatrixTest.isOperationMatrixScalar(op)) {
                            // Matrix-scalar \note For div cases we use uniform input.
                            var scalarIn = new es3fShaderMatrixTest.ShaderInput(op == es3fShaderMatrixTest.MatrixOp.OP_DIV ? es3fShaderMatrixTest.InputType.INPUTTYPE_UNIFORM : es3fShaderMatrixTest.InputType.INPUTTYPE_DYNAMIC, gluShaderUtil.DataType.FLOAT, precision);
                            inGroup.addChild(new es3fShaderMatrixTest.ShaderMatrixCase(baseName + 'float_vertex', 'Matrix-scalar case', matIn, scalarIn, op, true));
                            inGroup.addChild(new es3fShaderMatrixTest.ShaderMatrixCase(baseName + 'float_fragment',	'Matrix-scalar case', matIn, scalarIn, op, false));
                        }

                        if (es3fShaderMatrixTest.isOperationMatrixVector(op)) {
                            // Matrix-vector.
                            var colVecType = gluShaderUtil.getDataTypeFloatVec(numCols);
                            var colVecIn = new es3fShaderMatrixTest.ShaderInput(op == es3fShaderMatrixTest.MatrixOp.OP_DIV ? es3fShaderMatrixTest.InputType.INPUTTYPE_UNIFORM : es3fShaderMatrixTest.InputType.INPUTTYPE_DYNAMIC, colVecType, precision);

                            inGroup.addChild(new es3fShaderMatrixTest.ShaderMatrixCase(baseName + gluShaderUtil.getDataTypeName(colVecType) + '_vertex', 'Matrix-vector case', matIn, colVecIn, op, true));
                            inGroup.addChild(new es3fShaderMatrixTest.ShaderMatrixCase(baseName + gluShaderUtil.getDataTypeName(colVecType) + '_fragment', 'Matrix-vector case', matIn, colVecIn, op, false));

                            // Vector-matrix.
                            var rowVecType = gluShaderUtil.getDataTypeFloatVec(numRows);
                            var rowVecIn = new es3fShaderMatrixTest.ShaderInput(op == es3fShaderMatrixTest.MatrixOp.OP_DIV ? es3fShaderMatrixTest.InputType.INPUTTYPE_UNIFORM : es3fShaderMatrixTest.InputType.INPUTTYPE_DYNAMIC, rowVecType, precision);
                            var vecMatName = precName + '_' + gluShaderUtil.getDataTypeName(rowVecType) + '_' + matTypeName;

                            inGroup.addChild(new es3fShaderMatrixTest.ShaderMatrixCase(vecMatName + '_vertex', 'Vector-matrix case', rowVecIn, matIn, op, true));
                            inGroup.addChild(new es3fShaderMatrixTest.ShaderMatrixCase(vecMatName + '_fragment', 'Vector-matrix case', rowVecIn, matIn, op, false));
                        }

                        if (es3fShaderMatrixTest.isOperationArithmeticMatrixMatrix(op)) {
                            // Arithmetic matrix-matrix multiplication.
                            for (var otherCols = 2; otherCols <= 4; otherCols++) {
                                var otherMatIn = new es3fShaderMatrixTest.ShaderInput(inputType == es3fShaderMatrixTest.InputType.INPUTTYPE_DYNAMIC ? es3fShaderMatrixTest.InputType.INPUTTYPE_UNIFORM : inputType, gluShaderUtil.getDataTypeMatrix(otherCols, numCols), precision);
                                inGroup.addChild(new es3fShaderMatrixTest.ShaderMatrixCase(baseName + gluShaderUtil.getDataTypeName(otherMatIn.dataType) + '_vertex',	'Matrix-matrix case', matIn, otherMatIn, op, true));
                                inGroup.addChild(new es3fShaderMatrixTest.ShaderMatrixCase(baseName + gluShaderUtil.getDataTypeName(otherMatIn.dataType) + '_fragment',	'Matrix-matrix case', matIn, otherMatIn, op, false));
                            }
                        } else if (es3fShaderMatrixTest.isOperationComponentwiseMatrixMatrix(op)) {
                            // Component-wise.
                            var otherMatIn = new es3fShaderMatrixTest.ShaderInput(inputType == es3fShaderMatrixTest.InputType.INPUTTYPE_DYNAMIC ? es3fShaderMatrixTest.InputType.INPUTTYPE_UNIFORM : inputType, matType, precision);
                            inGroup.addChild(new es3fShaderMatrixTest.ShaderMatrixCase(baseName + matTypeName + '_vertex', 'Matrix-matrix case', matIn, otherMatIn, op, true));
                            inGroup.addChild(new es3fShaderMatrixTest.ShaderMatrixCase(baseName + matTypeName + '_fragment', 'Matrix-matrix case', matIn, otherMatIn, op, false));
                        }

                        if (es3fShaderMatrixTest.isOperationVectorVector(op)) {
                            var vec1In = new es3fShaderMatrixTest.ShaderInput(inputType, gluShaderUtil.getDataTypeFloatVec(numRows), precision);
                            var vec2In = new es3fShaderMatrixTest.ShaderInput((inputType == es3fShaderMatrixTest.InputType.INPUTTYPE_DYNAMIC) ? es3fShaderMatrixTest.InputType.INPUTTYPE_UNIFORM : inputType, gluShaderUtil.getDataTypeFloatVec(numCols), precision);

                            inGroup.addChild(new es3fShaderMatrixTest.ShaderMatrixCase(baseName + 'float_vertex', 'Vector-vector case', vec1In, vec2In, op, true));
                            inGroup.addChild(new es3fShaderMatrixTest.ShaderMatrixCase(baseName + 'float_fragment', 'Vector-vector case', vec1In, vec2In, op, false));
                        }

                        if (es3fShaderMatrixTest.isOperationUnaryAnyMatrix(op) || (es3fShaderMatrixTest.isOperationUnarySymmetricMatrix(op) && numCols == numRows)) {
                            var voidInput = new es3fShaderMatrixTest.ShaderInput();
                            inGroup.addChild(new es3fShaderMatrixTest.ShaderMatrixCase(baseName + 'float_vertex', 'Matrix case', matIn, voidInput, op, true));
                            inGroup.addChild(new es3fShaderMatrixTest.ShaderMatrixCase(baseName + 'float_fragment', 'Matrix case', matIn, voidInput, op, false));
                        }

                        if (es3fShaderMatrixTest.isOperationAssignmentAnyMatrix(op) || (es3fShaderMatrixTest.isOperationAssignmentSymmetricMatrix(op) && numCols == numRows)) {
                            var otherMatIn = new es3fShaderMatrixTest.ShaderInput(inputType == es3fShaderMatrixTest.InputType.INPUTTYPE_DYNAMIC ? es3fShaderMatrixTest.InputType.INPUTTYPE_UNIFORM : inputType, matType, precision);
                            inGroup.addChild(new es3fShaderMatrixTest.ShaderMatrixCase(baseName + 'float_vertex', 'Matrix assignment case', matIn, otherMatIn, op, true));
                            inGroup.addChild(new es3fShaderMatrixTest.ShaderMatrixCase(baseName + 'float_fragment', 'Matrix assignment case', matIn, otherMatIn, op, false));
                        }
                    }
                }
            }
        }
    }

    es3fShaderMatrixTest.run = function(context, range) {
        gl = context;
        //Set up Test Root parameters
        var testName = 'shader_matrix';
        var testDescription = 'Shader Matrix Test';
        var state = tcuTestCase.runner;

        state.testName = testName;
        state.testCases = tcuTestCase.newTest(testName, testDescription, null);

        //Set up name and description of this test series.
        setCurrentTestName(testName);
        description(testDescription);

        try {
            //Create test cases
            es3fShaderMatrixTest.init();
            if (range)
                state.setRange(range);
            //Run test cases
            tcuTestCase.runTestCases();
        }
        catch (err) {
            bufferedLogToConsole(err);
            tcuTestCase.runner.terminate();
        }
    };
});
