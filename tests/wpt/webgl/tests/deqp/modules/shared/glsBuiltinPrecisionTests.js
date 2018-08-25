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
goog.provide('modules.shared.glsBuiltinPrecisionTests');
goog.require('framework.common.tcuFloatFormat');
goog.require('framework.common.tcuInterval');
goog.require('framework.common.tcuMatrix');
goog.require('framework.common.tcuMatrixUtil');
goog.require('framework.common.tcuTestCase');
goog.require('framework.delibs.debase.deMath');
goog.require('framework.delibs.debase.deRandom');
goog.require('framework.delibs.debase.deUtil');
goog.require('framework.opengl.gluShaderProgram');
goog.require('framework.opengl.gluShaderUtil');
goog.require('framework.opengl.gluVarType');
goog.require('framework.opengl.simplereference.sglrGLContext');
goog.require('modules.shared.glsBuiltinPrecisionTestsUnitTests');
goog.require('modules.shared.glsShaderExecUtil');

goog.scope(function() {

    var glsBuiltinPrecisionTests = modules.shared.glsBuiltinPrecisionTests;
    var tcuTestCase = framework.common.tcuTestCase;
    var gluShaderProgram = framework.opengl.gluShaderProgram;
    var gluShaderUtil = framework.opengl.gluShaderUtil;
    var tcuInterval = framework.common.tcuInterval;
    var tcuFloatFormat = framework.common.tcuFloatFormat;
    var deRandom = framework.delibs.debase.deRandom;
    var glsShaderExecUtil = modules.shared.glsShaderExecUtil;
    var sglrGLContext = framework.opengl.simplereference.sglrGLContext;
    var deMath = framework.delibs.debase.deMath;
    var deUtil = framework.delibs.debase.deUtil;
    var gluVarType = framework.opengl.gluVarType;
    var tcuMatrix = framework.common.tcuMatrix;
    var tcuMatrixUtil = framework.common.tcuMatrixUtil;
    var ref = modules.shared.glsBuiltinPrecisionTestsUnitTests.cppreference;
    var referenceComparison = modules.shared.glsBuiltinPrecisionTestsUnitTests.referenceComparison;

    var DE_ASSERT = function(x) {
        if (!x)
            throw new Error('Assert failed');
    };

var setParentClass = function(child, parent) {
    child.prototype = Object.create(parent.prototype);
    child.prototype.constructor = child;
};

    /** @typedef {(tcuInterval.Interval|Array<tcuInterval.Interval>|tcuMatrix.Matrix)} */
    glsBuiltinPrecisionTests.Intervals;

    /** @typedef {(number|Array<number>|tcuMatrix.Matrix)} */
    glsBuiltinPrecisionTests.Value;

    /** @typedef {(string)} */
    glsBuiltinPrecisionTests.Typename;

    //Change to true for WebGL unit testing
    var enableUnittests = false;

    /**
     * @param {number} value
     * @return {boolean}
     */
    glsBuiltinPrecisionTests.isFloat = function(value) {
        return value % 1 !== 0;
     };

    /**
     * @constructor
     * @param {string} R
     * @param {string=} P0
     * @param {string=} P1
     * @param {string=} P2
     * @param {string=} P3
     */
    glsBuiltinPrecisionTests.Signature = function(R, P0, P1, P2, P3) {
        this.Ret = R;
        this.Arg0 = P0 === undefined ? 'void' : P0;
        this.Arg1 = P1 === undefined ? 'void' : P1;
        this.Arg2 = P2 === undefined ? 'void' : P2;
        this.Arg3 = P3 === undefined ? 'void' : P3;
    };

    /** @typedef {Array<glsBuiltinPrecisionTests.FuncBase>} */
    glsBuiltinPrecisionTests.FuncSet;

    /**
     * @constructor
     * @template T
     * @param {T} A0
     * @param {T} A1
     * @param {T} A2
     * @param {T} A3
     */
    glsBuiltinPrecisionTests.Tuple4 = function(A0, A1, A2, A3) {
        this.a = A0;
        this.b = A1;
        this.c = A2;
        this.d = A3;
    };

    /**
     * @typedef {!glsBuiltinPrecisionTests.Tuple4<string>}
     */
    glsBuiltinPrecisionTests.ParamNames;

    /**
     * Returns true for all other types except Void
     * @param {string} typename
     */
    glsBuiltinPrecisionTests.isTypeValid = function(typename) {
        if (typename === 'void')
            return false;
        return true;
    };

    /**
     * Returns true for all other types except Void
     * @param {*} In
     * @return {number}
     */
    glsBuiltinPrecisionTests.numInputs = function(In) {
        return (!glsBuiltinPrecisionTests.isTypeValid(In.In0) ? 0 :
                !glsBuiltinPrecisionTests.isTypeValid(In.In1) ? 1 :
                !glsBuiltinPrecisionTests.isTypeValid(In.In2) ? 2 :
                !glsBuiltinPrecisionTests.isTypeValid(In.In3) ? 3 :
                4);
    };

    /**
     * Returns true for all other types except Void
     * @param {*} Out
     * @return {number}
     */
    glsBuiltinPrecisionTests.numOutputs = function(Out) {
        return (!glsBuiltinPrecisionTests.isTypeValid(Out.Out0) ? 0 :
                !glsBuiltinPrecisionTests.isTypeValid(Out.Out1) ? 1 :
                2);
    };

    /**
     * @constructor
     * @param {glsBuiltinPrecisionTests.Typename=} In0_
     * @param {glsBuiltinPrecisionTests.Typename=} In1_
     * @param {glsBuiltinPrecisionTests.Typename=} In2_
     * @param {glsBuiltinPrecisionTests.Typename=} In3_
     */
    glsBuiltinPrecisionTests.InTypes = function(In0_, In1_, In2_, In3_) {
        this.In0 = In0_ === undefined ? 'void' : In0_;
        this.In1 = In1_ === undefined ? 'void' : In1_;
        this.In2 = In2_ === undefined ? 'void' : In2_;
        this.In3 = In3_ === undefined ? 'void' : In3_;
    };

    /**
     * @constructor
     * @param {glsBuiltinPrecisionTests.Typename=} Out0_
     * @param {glsBuiltinPrecisionTests.Typename=} Out1_
     */
    glsBuiltinPrecisionTests.OutTypes = function(Out0_, Out1_) {
        this.Out0 = Out0_ === undefined ? 'void' : Out0_;
        this.Out1 = Out1_ === undefined ? 'void' : Out1_;
    };

    /**
     * @constructor
     */
    glsBuiltinPrecisionTests.Environment = function() {
        /** @type {Object} */ this.m_map = {};
    };

    /**
     * @param {glsBuiltinPrecisionTests.Variable} variable
     * @param {*} value
     */
    glsBuiltinPrecisionTests.Environment.prototype.bind = function(variable, value) {
        this.m_map[variable.getName()] = value;
    };

    /**
     * @param {*} variable
     * @return {glsBuiltinPrecisionTests.Intervals}
     */
    glsBuiltinPrecisionTests.Environment.prototype.lookup = function(variable) {
        if (variable instanceof glsBuiltinPrecisionTests.Variable)
           return this.m_map[variable.getName()];

        throw new Error('Invalid lookup input: ' + variable);
    };

    /**
     * @constructor
     * @param {tcuFloatFormat.FloatFormat} format_
     * @param {gluShaderUtil.precision} floatPrecision_
     * @param {glsBuiltinPrecisionTests.Environment} env_
     * @param {number=} callDepth_
     */
    glsBuiltinPrecisionTests.EvalContext = function(format_, floatPrecision_, env_, callDepth_) {
        this.format = format_;
        this.floatPrecision = floatPrecision_;
        this.env = env_;
        this.callDepth = callDepth_ === undefined ? 0 : callDepth_;
    };

    /**
     * @param {string} typename typename
     * @param {tcuFloatFormat.FloatFormat} fmt
     * @param {glsBuiltinPrecisionTests.Intervals} value
     * @return {glsBuiltinPrecisionTests.Intervals}
     */
     glsBuiltinPrecisionTests.convert = function(typename, fmt, value) {
        var traits = glsBuiltinPrecisionTests.Traits.traitsFactory(typename);

        if (value instanceof Array) {
            var ret = [];
            for (var i = 0; i < value.length; i++)
                ret.push(traits.doConvert(fmt, value[i]));
            return ret;
        }

        if (value instanceof tcuMatrix.Matrix) {
            var ret = new tcuMatrix.Matrix(value.rows, value.cols);
            for (var i = 0; i < value.rows; i++)
                for (var j = 0; j < value.cols; j++)
                    ret.set(i, j, traits.doConvert(fmt, value.get(i, j)));
            return ret;
        }

        return traits.doConvert(fmt, value);
    };

    /**
     * Returns true if every element of `ival` contains the corresponding element of `value`.
     * @param {string} typename typename
     * @param {glsBuiltinPrecisionTests.Intervals} ival
     * @param {*} value
     * @return {boolean}
     */
     glsBuiltinPrecisionTests.contains = function(typename, ival, value) {
        var traits = glsBuiltinPrecisionTests.Traits.traitsFactory(typename);
        var contains = true;

        if (value instanceof Array) {
            for (var i = 0; i < value.length; i++)
                contains &= traits.doContains(ival[i], value[i]);
            return contains;
        }

        if (value instanceof tcuMatrix.Matrix) {
            for (var i = 0; i < value.rows; i++)
                for (var j = 0; j < value.cols; j++)
                    contains &= traits.doContains(ival.get(i, j), value.get(i, j));
            return contains;
        }

        return traits.doContains(ival, value);
    };

    /**
     * @param {string} typename typename
     * @param {glsBuiltinPrecisionTests.Intervals} ival0
     * @param {glsBuiltinPrecisionTests.Intervals} ival1
     * @return {glsBuiltinPrecisionTests.Intervals}
     */
     glsBuiltinPrecisionTests.union = function(typename, ival0, ival1) {
        var traits = glsBuiltinPrecisionTests.Traits.traitsFactory(typename);

        if (ival0 instanceof Array) {
            var ret = [];
            for (var i = 0; i < ival0.length; i++)
                ret.push(traits.doUnion(ival0[i], ival1[i]));
            return ret;
        }

        if (ival0 instanceof tcuMatrix.Matrix) {
            var ret = new tcuMatrix.Matrix(ival0.rows, ival0.cols);
            for (var i = 0; i < ival0.rows; i++)
                for (var j = 0; j < ival0.cols; j++)
                    ret.set(i, j, traits.doUnion(ival0.get(i, j), ival1.get(i, j)));
            return ret;
        }

        return traits.doUnion(ival0, ival1);
    };

    /**
     * @param {string} typename
     * @constructor
     */
    glsBuiltinPrecisionTests.Traits = function(typename) {
        this.typename = typename;
        this.rows = 1;
        this.cols = 1;
    };

    glsBuiltinPrecisionTests.Traits.prototype.isScalar = function() {
        return this.rows == 1 && this.cols == 1;
    };

    glsBuiltinPrecisionTests.Traits.prototype.isVector = function() {
        return this.rows > 0 && this.cols == 1;
    };

    glsBuiltinPrecisionTests.Traits.prototype.isMatrix = function() {
        return this.rows > 0 && this.cols > 1;
    };

    /**
     * @param {string=} typename
     */
    glsBuiltinPrecisionTests.Traits.traitsFactory = function(typename) {
        switch (typename) {
            case 'boolean' : return new glsBuiltinPrecisionTests.TraitsBool();
            case 'float' : case 'vec2' : case 'vec3' : case 'vec4' :
            case 'mat2' : case 'mat2x3' : case 'mat2x4' :
            case 'mat3x2' : case 'mat3' : case 'mat3x4' :
            case 'mat4x2' : case 'mat4x3' : case 'mat4' :
                return new glsBuiltinPrecisionTests.TraitsFloat(typename);
            case 'int' : return new glsBuiltinPrecisionTests.TraitsInt();
            case 'void' : return new glsBuiltinPrecisionTests.TraitsVoid();
            default:
                throw new Error('Invalid typename:' + typename);
        }
    };

    glsBuiltinPrecisionTests.round = function(typename, fmt, value) {
        var traits = glsBuiltinPrecisionTests.Traits.traitsFactory(typename);

        if (value instanceof Array) {
            var ret = [];
            for (var i = 0; i < value.length; i++)
                ret.push(traits.doRound(fmt, value[i]));
            return ret;
        }

        if (value instanceof tcuMatrix.Matrix) {
            var ret = new tcuMatrix.Matrix(value.rows, value.cols);
            for (var i = 0; i < value.rows; i++)
                for (var j = 0; j < value.cols; j++)
                    ret.set(i, j, traits.doRound(fmt, value.get(i, j)));
            return ret;
        }

        return traits.doRound(fmt, value);
    };

    /**
     * cast the input typed array to correct type
     * @param {string} typename
     * @param {goog.TypedArray} input
     * @return {goog.TypedArray}
     */
    glsBuiltinPrecisionTests.cast = function(typename, input) {
        var traits = glsBuiltinPrecisionTests.Traits.traitsFactory(typename);
        return traits.doCast(input);
    };

    /**
     * @constructor
     * @extends {glsBuiltinPrecisionTests.Traits}
     */
    glsBuiltinPrecisionTests.TraitsVoid = function() {
        glsBuiltinPrecisionTests.Traits.call(this, 'void');
    };

    setParentClass(glsBuiltinPrecisionTests.TraitsVoid, glsBuiltinPrecisionTests.Traits);

    /**
     * @param {*} value
     * @return {tcuInterval.Interval}
     */
    glsBuiltinPrecisionTests.TraitsVoid.prototype.doMakeIVal = function(value) {
        return new tcuInterval.Interval();
    };

    /**
     * @param {*} value1
     * @param {*} value2
     * @return {tcuInterval.Interval}
     */
    glsBuiltinPrecisionTests.TraitsVoid.prototype.doUnion = function(value1, value2) {
        return new tcuInterval.Interval();
    };

    /**
     * @param {*} value
     * @return {boolean}
     */
    glsBuiltinPrecisionTests.TraitsVoid.prototype.doContains = function(value) {
        return true;
    };

    /**
     * @param {tcuFloatFormat.FloatFormat} fmt
     * @param {tcuInterval.Interval} ival
     * @return {tcuInterval.Interval}
     */
    glsBuiltinPrecisionTests.TraitsVoid.prototype.doConvert = function(fmt, ival) {
        return new tcuInterval.Interval();
    };

    /**
     * @param {tcuFloatFormat.FloatFormat} fmt
     * @param {*} ival
     * @return {tcuInterval.Interval}
     */
    glsBuiltinPrecisionTests.TraitsVoid.prototype.doRound = function(fmt, ival) {
        return new tcuInterval.Interval();
    };

    /**
     * @param {tcuFloatFormat.FloatFormat} fmt
     * @param {*} ival
     */
    glsBuiltinPrecisionTests.TraitsVoid.prototype.doPrintIVal = function(fmt, ival) {
        return '()';
    };

    /**
     * @param {tcuFloatFormat.FloatFormat} fmt
     * @param {*} value
     */
    glsBuiltinPrecisionTests.TraitsVoid.prototype.doPrintValue = function(fmt, value) {
        return '()';
    };

    glsBuiltinPrecisionTests.dataTypeSize = function(detailedType) {
        var size = [1, 1];
        switch (detailedType) {
            case 'vec2' : size[0] = 2; break;
            case 'vec3' : size[0] = 3; break;
            case 'vec4' : size[0] = 4; break;
            case 'mat2' : size = [2 , 2]; break;
            case 'mat2x3' : size = [3 , 2]; break;
            case 'mat2x4' : size = [4 , 2]; break;

            case 'mat3x2' : size = [2 , 3]; break;
            case 'mat3' : size = [3 , 3]; break;
            case 'mat3x4' : size = [4 , 3]; break;

            case 'mat4x2' : size = [2 , 4]; break;
            case 'mat4x3' : size = [3 , 4]; break;
            case 'mat4' : size = [4 , 4]; break;
        }
        return size;
    };

    /**
     * @constructor
     * @extends {glsBuiltinPrecisionTests.Traits}
     * @param {string} typename
     * @param {string=} detailedType
     */
    glsBuiltinPrecisionTests.ScalarTraits = function(typename, detailedType) {
        glsBuiltinPrecisionTests.Traits.call(this, typename);
        var size = glsBuiltinPrecisionTests.dataTypeSize(detailedType);
        this.rows = size[0];
        this.cols = size[1];

        /** type{tcuInterval.Interval} */ this.iVal;
    };

    setParentClass(glsBuiltinPrecisionTests.ScalarTraits, glsBuiltinPrecisionTests.Traits);

    glsBuiltinPrecisionTests.ScalarTraits.prototype = Object.create(glsBuiltinPrecisionTests.Traits.prototype);
    glsBuiltinPrecisionTests.ScalarTraits.prototype.constructor = glsBuiltinPrecisionTests.ScalarTraits;

    /**
     * @param {*} value
     * @return {tcuInterval.Interval}
     */
    glsBuiltinPrecisionTests.ScalarTraits.prototype.doMakeIVal = function(value) {
        // Thankfully all scalar types have a well-defined conversion to `double`,
        // hence Interval can represent their ranges without problems.
        return new tcuInterval.Interval(/** @type {number} */ (value));
    };

    /**
     * @param {tcuInterval.Interval} a
     * @param {tcuInterval.Interval} b
     * @return {tcuInterval.Interval}
     */
    glsBuiltinPrecisionTests.ScalarTraits.prototype.doUnion = function(a, b) {
        return a.operatorOrBinary(b);
    };

    /**
     * @param {tcuInterval.Interval} a
     * @param {number} value
     * @return {boolean}
     */
    glsBuiltinPrecisionTests.ScalarTraits.prototype.doContains = function(a, value) {
        return a.contains(new tcuInterval.Interval(value));
    };

    /**
     * @param {tcuFloatFormat.FloatFormat} fmt
     * @param {tcuInterval.Interval} ival
     * @return {tcuInterval.Interval}
     */
    glsBuiltinPrecisionTests.ScalarTraits.prototype.doConvert = function(fmt, ival) {
        return fmt.convert(ival);
    };

    /**
     * @param {tcuFloatFormat.FloatFormat} fmt
     * @param {number} value
     * @return {tcuInterval.Interval}
     */
    glsBuiltinPrecisionTests.ScalarTraits.prototype.doRound = function(fmt, value) {
        return fmt.roundOut(new tcuInterval.Interval(value), false);//TODO cast to double
    };

    /**
     * @constructor
     * @extends {glsBuiltinPrecisionTests.ScalarTraits}
     * @param {string} detailedType
     */
    glsBuiltinPrecisionTests.TraitsFloat = function(detailedType) {
        glsBuiltinPrecisionTests.ScalarTraits.call(this, 'float', detailedType);
    };

    glsBuiltinPrecisionTests.TraitsFloat.prototype = Object.create(glsBuiltinPrecisionTests.ScalarTraits.prototype);
    glsBuiltinPrecisionTests.TraitsFloat.prototype.constructor = glsBuiltinPrecisionTests.TraitsFloat;

    /**
     * @param {tcuFloatFormat.FloatFormat} fmt
     * @param {tcuInterval.Interval} ival
     */
    glsBuiltinPrecisionTests.TraitsFloat.prototype.doPrintIVal = function(fmt, ival) {
        return fmt.intervalToHex(ival);
    };

    /**
     * @param {goog.TypedArray} input
     * @return {goog.TypedArray}
     */
    glsBuiltinPrecisionTests.TraitsFloat.prototype.doCast = function(input) {
        return new Float32Array(input.buffer);
    };

    /**
     * @param {tcuFloatFormat.FloatFormat} fmt
     * @param {number} value
     */
    glsBuiltinPrecisionTests.TraitsFloat.prototype.doPrintValue = function(fmt, value) {
        return fmt.floatToHex(value);
    };

    /**
     * @constructor
     * @extends {glsBuiltinPrecisionTests.ScalarTraits}
     */
    glsBuiltinPrecisionTests.TraitsBool = function() {
        glsBuiltinPrecisionTests.ScalarTraits.call(this, 'boolean');
    };

    glsBuiltinPrecisionTests.TraitsBool.prototype = Object.create(glsBuiltinPrecisionTests.ScalarTraits.prototype);
    glsBuiltinPrecisionTests.TraitsBool.prototype.constructor = glsBuiltinPrecisionTests.TraitsBool;

    /**
     * @param {tcuFloatFormat.FloatFormat} fmt
     * @param {tcuInterval.Interval} ival
     */
    glsBuiltinPrecisionTests.TraitsBool.prototype.doPrintIVal = function(fmt, ival) {
        /** type{string} */ var os = '{';
        var ifalse = new tcuInterval.Interval(0);
        var itrue = new tcuInterval.Interval(1);
        if (ival.contains(ifalse))
            os += 'false';
        if (ival.contains(ifalse) && ival.contains(itrue))
            os += ', ';
        if (ival.contains(itrue))
            os += 'true';
        os += '}';
        return os;
    };

    /**
     * @param {tcuFloatFormat.FloatFormat} fmt
     * @param {boolean} value
     */
    glsBuiltinPrecisionTests.TraitsBool.prototype.doPrintValue = function(fmt, value) {
        return value ? 'true' : 'false';
    };

    /**
     * @constructor
     * @extends {glsBuiltinPrecisionTests.ScalarTraits}
     */
    glsBuiltinPrecisionTests.TraitsInt = function() {
        glsBuiltinPrecisionTests.ScalarTraits.call(this, 'int');
    };

    glsBuiltinPrecisionTests.TraitsInt.prototype = Object.create(glsBuiltinPrecisionTests.ScalarTraits.prototype);
    glsBuiltinPrecisionTests.TraitsInt.prototype.constructor = glsBuiltinPrecisionTests.TraitsInt;

    /**
     * @param {tcuFloatFormat.FloatFormat} fmt
     * @param {tcuInterval.Interval} ival
     */
    glsBuiltinPrecisionTests.TraitsInt.prototype.doPrintIVal = function(fmt, ival) {
        return '[' + (ival.lo()) + ', ' + (ival.hi()) + ']';
    };

    /**
     * @param {tcuFloatFormat.FloatFormat} fmt
     * @param {number} value
     */
    glsBuiltinPrecisionTests.TraitsInt.prototype.doPrintValue = function(fmt, value) {
        return value.toString(10);
    };

    /**
     * @constructor
     */
    glsBuiltinPrecisionTests.Statement = function() {

    };

    /**
     * @param {glsBuiltinPrecisionTests.EvalContext} ctx
     */
    glsBuiltinPrecisionTests.Statement.prototype.execute = function(ctx) {
        this.doExecute(ctx);
    };

    /**
     * @return {string}
     */
    glsBuiltinPrecisionTests.Statement.prototype.print = function() {
        return this.doPrint();
    };

    glsBuiltinPrecisionTests.Statement.prototype.toString = function() {
        return this.print();
    };

    /**
     * Output the functions that this expression refers to
     * @param {glsBuiltinPrecisionTests.FuncSet} dst
     *
     */
    glsBuiltinPrecisionTests.Statement.prototype.getUsedFuncs = function(dst) {
        this.doGetUsedFuncs(dst);
    };

    /**
     * @param {glsBuiltinPrecisionTests.EvalContext} ctx
     */
    glsBuiltinPrecisionTests.Statement.prototype.doExecute = function(ctx) {
        throw new Error('Virtual function. Please override.');
    };

    /**
     * @return {string}
     */
    glsBuiltinPrecisionTests.Statement.prototype.doPrint = function() {
        throw new Error('Virtual function. Please override.');
    };

    /**
     * Output the functions that this expression refers to
     * @param {glsBuiltinPrecisionTests.FuncSet} dst
     *
     */
    glsBuiltinPrecisionTests.Statement.prototype.doGetUsedFuncs = function(dst) {
        throw new Error('Virtual function. Please override.');
    };

    /**
     * @constructor
     * @extends {glsBuiltinPrecisionTests.Statement}
     * @param {glsBuiltinPrecisionTests.Variable} variable
     * @param {glsBuiltinPrecisionTests.Expr} value
     * @param {boolean} isDeclaration
     */
    glsBuiltinPrecisionTests.VariableStatement = function(variable, value, isDeclaration) {
        this.m_variable = variable;
        this.m_value = value;
        this.m_isDeclaration = isDeclaration;

    };

    setParentClass(glsBuiltinPrecisionTests.VariableStatement, glsBuiltinPrecisionTests.Statement);

    /**
     * @param {glsBuiltinPrecisionTests.EvalContext} ctx
     */
    glsBuiltinPrecisionTests.VariableStatement.prototype.doExecute = function(ctx) {
        ctx.env.bind(this.m_variable, this.m_value.evaluate(ctx));
    };

    /**
     * @return {string}
     */
    glsBuiltinPrecisionTests.VariableStatement.prototype.doPrint = function() {
        var v = this.m_variable;
        var os = '';
        if (this.m_isDeclaration)
            os += gluVarType.declareVariable(gluVarType.getVarTypeOf(v.typename),
                        v.getName());
        else
            os += v.getName();

        os += ' = ' + this.m_value.printExpr() + ';\n';

        return os;
    };

    /**
     * Output the functions that this expression refers to
     * @param {glsBuiltinPrecisionTests.FuncSet} dst
     *
     */
    glsBuiltinPrecisionTests.VariableStatement.prototype.doGetUsedFuncs = function(dst) {
        this.m_value.getUsedFuncs(dst);
    };

    /**
     * @param {glsBuiltinPrecisionTests.Variable} variable
     * @param {glsBuiltinPrecisionTests.Expr} definiens
     * @return {glsBuiltinPrecisionTests.VariableStatement}
     */
    glsBuiltinPrecisionTests.variableDeclaration = function(variable, definiens) {
        return new glsBuiltinPrecisionTests.VariableStatement(variable, definiens, true);
    };

    /**
     * @param {string} typename
     * @param {string} name
     * @param {glsBuiltinPrecisionTests.ExpandContext} ctx
     * @param {glsBuiltinPrecisionTests.Expr} expr
     * @return {glsBuiltinPrecisionTests.Variable}
     */
    glsBuiltinPrecisionTests.bindExpression = function(typename, name, ctx, expr) {
        var variable = ctx.genSym(typename, name);
        ctx.addStatement(glsBuiltinPrecisionTests.variableDeclaration(variable, expr));
        return variable;
    };

    /**
     * Common base class for all expressions regardless of their type.
     * @constructor
     */
    glsBuiltinPrecisionTests.ExprBase = function() {};

    /**
     * @return {string}
     */
    glsBuiltinPrecisionTests.ExprBase.prototype.printExpr = function() {
        return this.doPrintExpr();
    };

    glsBuiltinPrecisionTests.ExprBase.prototype.toString = function() {
        return this.printExpr();
    };

    /**
     * @return {string}
     */
    glsBuiltinPrecisionTests.ExprBase.prototype.doPrintExpr = function() {
        throw new Error('Virtual function. Please override.');
    };

    /**
     * Output the functions that this expression refers to
     * @param {glsBuiltinPrecisionTests.FuncSet} dst
     *
     */
    glsBuiltinPrecisionTests.ExprBase.prototype.getUsedFuncs = function(/*FuncSet&*/ dst) {
        this.doGetUsedFuncs(dst);
    };

    /**
     * Output the functions that this expression refers to
     * @param {glsBuiltinPrecisionTests.FuncSet} dst
     *
     */
    glsBuiltinPrecisionTests.ExprBase.prototype.doGetUsedFuncs = function(/*FuncSet&*/ dst) {
        throw new Error('Virtual function. Please override.');
    };

    /**
     * Type-specific operations for an expression representing type typename.
     * @constructor
     * @extends {glsBuiltinPrecisionTests.ExprBase}
     * @param {glsBuiltinPrecisionTests.Typename} typename
     */
    glsBuiltinPrecisionTests.Expr = function(typename) {
        glsBuiltinPrecisionTests.ExprBase.call(this);
        this.typename = typename;
    };

    setParentClass(glsBuiltinPrecisionTests.Expr, glsBuiltinPrecisionTests.ExprBase);

    /**
     * Type-specific operations for an expression representing type typename.
     * @param {glsBuiltinPrecisionTests.EvalContext} ctx
     */
    glsBuiltinPrecisionTests.Expr.prototype.evaluate = function(ctx) {
        return this.doEvaluate(ctx);
    };

    /**
     * Type-specific operations for an expression representing type typename.
     * @param {glsBuiltinPrecisionTests.EvalContext} ctx
     */
    glsBuiltinPrecisionTests.Expr.prototype.doEvaluate = function(ctx) {
        throw new Error('Virtual function. Please override.');
    };

    /**
     * @constructor
     * @extends {glsBuiltinPrecisionTests.Expr}
     * @param {glsBuiltinPrecisionTests.Typename} typename
     * @param {string=} name
     */
    glsBuiltinPrecisionTests.Variable = function(typename, name) {
        glsBuiltinPrecisionTests.Expr.call(this, typename);
        /** @type {string} */ this.m_name = name || '<undefined>';
    };

    setParentClass(glsBuiltinPrecisionTests.Variable, glsBuiltinPrecisionTests.Expr);

    /**
     * @return {string}
     */
    glsBuiltinPrecisionTests.Variable.prototype.getName = function() {
        return this.m_name;
    };

    /**
     * @return {string}
     */
    glsBuiltinPrecisionTests.Variable.prototype.doPrintExpr = function() {
        return this.m_name;
    };

    glsBuiltinPrecisionTests.Variable.prototype.toString = function() {
        return this.doPrintExpr();
    };

    /**
     * @param {glsBuiltinPrecisionTests.EvalContext} ctx
     * @return {*}
     */
    glsBuiltinPrecisionTests.Variable.prototype.doEvaluate = function(ctx) {
        return ctx.env.lookup(this);
    };

    /**
     * @constructor
     * @extends {glsBuiltinPrecisionTests.Variable}
     * @param {*=} t
     */
    glsBuiltinPrecisionTests.Void = function(t) {
        glsBuiltinPrecisionTests.Variable.call(this, 'void');
    };

    setParentClass(glsBuiltinPrecisionTests.Void, glsBuiltinPrecisionTests.Variable);

    glsBuiltinPrecisionTests.Void.prototype.doEvaluate = function(ctx) {
        return undefined;
    };

    /**
     * @constructor
     * @extends {glsBuiltinPrecisionTests.Variable}
     * @param {number} value
     */
    glsBuiltinPrecisionTests.Constant = function(value) {
        glsBuiltinPrecisionTests.Variable.call(this, 'float');
        this.m_value = value;
    };

    setParentClass(glsBuiltinPrecisionTests.Constant, glsBuiltinPrecisionTests.Variable);

    glsBuiltinPrecisionTests.Constant.prototype.doEvaluate = function(ctx) {
        return new tcuInterval.Interval(this.m_value);
    };

    /**
     * @constructor
     * @param {*} typename
     */
    glsBuiltinPrecisionTests.DefaultSampling = function(typename) {
        this.typename = typename;
    };

    /**
     * @constructor
     * @extends {glsBuiltinPrecisionTests.Expr}
     * @param {glsBuiltinPrecisionTests.Variable} vector
     * @param {number} index
     */
    glsBuiltinPrecisionTests.VectorVariable = function(vector, index) {
        glsBuiltinPrecisionTests.Expr.call(this, vector.typename);
        this.m_vector = vector;
        this.m_index = index;
    };

    setParentClass(glsBuiltinPrecisionTests.VectorVariable, glsBuiltinPrecisionTests.Expr);

    /**
     * @param {glsBuiltinPrecisionTests.EvalContext} ctx
     * @return {tcuInterval.Interval}
     */
    glsBuiltinPrecisionTests.VectorVariable.prototype.doEvaluate = function(ctx) {
        var tmp = this.m_vector.doEvaluate(ctx);
        return tmp[this.m_index];
    };

    /**
     * @constructor
     * @extends {glsBuiltinPrecisionTests.Expr}
     * @param {glsBuiltinPrecisionTests.Variable} matrix
     * @param {number} row
     * @param {number} col
     */
    glsBuiltinPrecisionTests.MatrixVariable = function(matrix, row, col) {
        glsBuiltinPrecisionTests.Expr.call(this, matrix.typename);
        this.m_matrix = matrix;
        this.m_row = row;
        this.m_col = col;
    };

    setParentClass(glsBuiltinPrecisionTests.MatrixVariable, glsBuiltinPrecisionTests.Expr);

    /**
     * @param {glsBuiltinPrecisionTests.EvalContext} ctx
     * @return {tcuInterval.Interval}
     */
    glsBuiltinPrecisionTests.MatrixVariable.prototype.doEvaluate = function(ctx) {
        var tmp = this.m_matrix.doEvaluate(ctx);
        return tmp.get(this.m_row, this.m_col);
    };

    /**
     * @constructor
     * @extends {glsBuiltinPrecisionTests.Expr}
     * @param {glsBuiltinPrecisionTests.Typename} typename
     * @param {glsBuiltinPrecisionTests.Func} func
     * @param {glsBuiltinPrecisionTests.Expr=} arg0
     * @param {glsBuiltinPrecisionTests.Expr=} arg1
     * @param {glsBuiltinPrecisionTests.Expr=} arg2
     * @param {glsBuiltinPrecisionTests.Expr=} arg3
     */
    glsBuiltinPrecisionTests.Apply = function(typename, func, arg0, arg1, arg2, arg3) {
        glsBuiltinPrecisionTests.Expr.call(this, typename);
        this.m_func = func;
        /** @type {glsBuiltinPrecisionTests.Tuple4} */ this.m_args;
        if (arg0 instanceof glsBuiltinPrecisionTests.Tuple4)
            this.m_args = /** @type {glsBuiltinPrecisionTests.Tuple4} */ (arg0);
        else {
            this.m_args = new glsBuiltinPrecisionTests.Tuple4(arg0 || new glsBuiltinPrecisionTests.Void(),
                                                              arg1 || new glsBuiltinPrecisionTests.Void(),
                                                              arg2 || new glsBuiltinPrecisionTests.Void(),
                                                              arg3 || new glsBuiltinPrecisionTests.Void());
        }
    };

    setParentClass(glsBuiltinPrecisionTests.Apply, glsBuiltinPrecisionTests.Expr);

    /**
     * @return {string}
     */
    glsBuiltinPrecisionTests.Apply.prototype.doPrintExpr = function() {
        var args = [this.m_args.a, this.m_args.b, this.m_args.c, this.m_args.d];
        return this.m_func.print(args);
    };

    /**
     * @param {glsBuiltinPrecisionTests.EvalContext} ctx
     * @return {glsBuiltinPrecisionTests.Intervals}
     */
    glsBuiltinPrecisionTests.Apply.prototype.doEvaluate = function(ctx) {
        var debug = false;

        if (debug) {
            glsBuiltinPrecisionTests.Apply.prototype.doEvaluate.level = glsBuiltinPrecisionTests.Apply.prototype.doEvaluate.level || 0;
            var level = glsBuiltinPrecisionTests.Apply.prototype.doEvaluate.level;
            glsBuiltinPrecisionTests.Apply.prototype.doEvaluate.level++;
            var name = this.m_func.constructor.toString();
            name = name.replace(/[\s\S]*glsBuiltinPrecisionTests\./m, '').replace(/\.call[\s\S]*/m, '');
            if (this.m_func.getName)
                name += ' ' + this.m_func.getName();
            console.log('<' + level + '> Function ' + name);
        }

        var a = this.m_args.a.evaluate(ctx);
        var b = this.m_args.b.evaluate(ctx);
        var c = this.m_args.c.evaluate(ctx);
        var d = this.m_args.d.evaluate(ctx);
        var retVal = this.m_func.applyFunction(ctx, a, b, c, d);

        if (debug) {
            console.log('<' + level + '> a: ' + a);
            console.log('<' + level + '> b: ' + b);
            console.log('<' + level + '> returning: ' + retVal);
            glsBuiltinPrecisionTests.Apply.prototype.doEvaluate.level--;
        }
        return retVal;
    };

    /**
     * @param {glsBuiltinPrecisionTests.Func} func
     * @param {glsBuiltinPrecisionTests.Expr=} arg0
     * @param {glsBuiltinPrecisionTests.Expr=} arg1
     * @param {glsBuiltinPrecisionTests.Expr=} arg2
     * @param {glsBuiltinPrecisionTests.Expr=} arg3
     */
    var app = function(func, arg0, arg1, arg2, arg3) {
        return new glsBuiltinPrecisionTests.Apply('float', func, arg0, arg1, arg2, arg3);
    };

    /**
     * @param {glsBuiltinPrecisionTests.FuncSet} dst
     */
    glsBuiltinPrecisionTests.Apply.prototype.doGetUsedFuncs = function(dst) {
        this.m_func.getUsedFuncs(dst);
        this.m_args.a.getUsedFuncs(dst);
        this.m_args.b.getUsedFuncs(dst);
        this.m_args.c.getUsedFuncs(dst);
        this.m_args.d.getUsedFuncs(dst);
    };

    /**
     * @constructor
     * @extends {glsBuiltinPrecisionTests.Apply}
     * @param {glsBuiltinPrecisionTests.Func} func
     * @param {glsBuiltinPrecisionTests.Expr=} arg0
     * @param {glsBuiltinPrecisionTests.Expr=} arg1
     * @param {glsBuiltinPrecisionTests.Expr=} arg2
     * @param {glsBuiltinPrecisionTests.Expr=} arg3
     */
    glsBuiltinPrecisionTests.ApplyScalar = function(func, arg0, arg1, arg2, arg3) {
        glsBuiltinPrecisionTests.Apply.call(this, 'float', func, arg0, arg1, arg2, arg3);
    };

    setParentClass(glsBuiltinPrecisionTests.ApplyScalar, glsBuiltinPrecisionTests.Apply);

    glsBuiltinPrecisionTests.ApplyScalar.prototype.doEvaluate = function(ctx) {
        var debug = false;

        if (debug) {
            glsBuiltinPrecisionTests.ApplyScalar.prototype.doEvaluate.level = glsBuiltinPrecisionTests.ApplyScalar.prototype.doEvaluate.level || 0;
            var level = glsBuiltinPrecisionTests.ApplyScalar.prototype.doEvaluate.level;
            glsBuiltinPrecisionTests.ApplyScalar.prototype.doEvaluate.level++;
            var name = this.m_func.constructor.toString();
            name = name.replace(/[\s\S]*glsBuiltinPrecisionTests\./m, '').replace(/\.call[\s\S]*/m, '');
            if (this.m_func.getName)
                name += ' ' + this.m_func.getName();
            console.log('scalar<' + level + '> Function ' + name);
        }

        var a = this.m_args.a.evaluate(ctx);
        var b = this.m_args.b.evaluate(ctx);
        var c = this.m_args.c.evaluate(ctx);
        var d = this.m_args.d.evaluate(ctx);
        if (a instanceof Array) {
            var ret = [];
            for (var i = 0; i < a.length; i++) {
                var p0 = a instanceof Array ? a[i] : a;
                var p1 = b instanceof Array ? b[i] : b;
                var p2 = c instanceof Array ? c[i] : c;
                var p3 = d instanceof Array ? d[i] : d;
                ret.push(this.m_func.applyFunction(ctx, p0, p1, p2, p3));
            }
            return ret;
        }

        var retVal = this.m_func.applyFunction(ctx, a, b, c, d);

        if (debug) {
            console.log('scalar<' + level + '> a: ' + a);
            console.log('scalar<' + level + '> b: ' + b);
            console.log('scalar<' + level + '> return1: ' + ret);
            console.log('scalar<' + level + '> return2: ' + retVal);
            glsBuiltinPrecisionTests.Apply.prototype.doEvaluate.level--;
        }

        return retVal;
    };

    /**
     * @constructor
     * @extends {glsBuiltinPrecisionTests.Apply}
     */
    glsBuiltinPrecisionTests.ApplyVar = function(typename, func, arg0, arg1, arg2, arg3) {
        glsBuiltinPrecisionTests.Apply.call(this, typename, func, arg0, arg1, arg2, arg3);
    };

    setParentClass(glsBuiltinPrecisionTests.ApplyVar, glsBuiltinPrecisionTests.Apply);

    /**
     * @param {glsBuiltinPrecisionTests.EvalContext} ctx
     * @return {glsBuiltinPrecisionTests.Intervals}
     */
    glsBuiltinPrecisionTests.ApplyVar.prototype.doEvaluate = function(ctx) {
        return this.m_func.applyFunction(ctx,
                    ctx.env.lookup(this.m_args.a), ctx.env.lookup(this.m_args.b),
                    ctx.env.lookup(this.m_args.c), ctx.env.lookup(this.m_args.d),
                    [this.m_args.a.getName(), this.m_args.b.getName(),
                    this.m_args.c.getName(), this.m_args.d.getName()]);
    };

    /**
     * @constructor
     */
    glsBuiltinPrecisionTests.FuncBase = function() {};

    /**
     * @return {string}
     */
    glsBuiltinPrecisionTests.FuncBase.prototype.getName = function() {
        return '';
    };

    /**
     * @return {string}
     */
    glsBuiltinPrecisionTests.FuncBase.prototype.getRequiredExtension = function() {
        return '';
    };

    /**
     * @param {Array<glsBuiltinPrecisionTests.ExprBase>} args
     * @return {string}
     */
    glsBuiltinPrecisionTests.FuncBase.prototype.print = function(args) {
        return '';
    };

    /**
     * Index of output parameter, or -1 if none of the parameters is output.
     * @return {number}
     */
    glsBuiltinPrecisionTests.FuncBase.prototype.getOutParamIndex = function() {
        return -1;
    };

    /**
     * @return {string}
     */
    glsBuiltinPrecisionTests.FuncBase.prototype.printDefinition = function() {
        return this.doPrintDefinition();
    };

    /**
     * @return {string}
     */
    glsBuiltinPrecisionTests.FuncBase.prototype.doPrintDefinition = function() {
        throw new Error('Virtual function. Please override.');
    };

    /**
     * typedef set<const FuncBase*> FuncSet;
     * @param {glsBuiltinPrecisionTests.FuncSet} dst
     */
    glsBuiltinPrecisionTests.FuncBase.prototype.getUsedFuncs = function(dst) {
        this.doGetUsedFuncs(dst);
    };

    /**
     * @param {glsBuiltinPrecisionTests.FuncSet} dst
     */
    glsBuiltinPrecisionTests.FuncBase.prototype.doGetUsedFuncs = function(dst) {};

    /*************************************/
    /**
     * \brief Function objects.
     *
     * Each Func object represents a GLSL function. It can be applied to interval
     * arguments, and it returns the an interval that is a conservative
     * approximation of the image of the GLSL function over the argument
     * intervals. That is, it is given a set of possible arguments and it returns
     * the set of possible values.
     *
     * @constructor
     * @extends {glsBuiltinPrecisionTests.FuncBase}
     * @param {glsBuiltinPrecisionTests.Signature} Sig_ template <typename Sig_>
     */
    glsBuiltinPrecisionTests.Func = function(Sig_) {
        glsBuiltinPrecisionTests.FuncBase.call(this);
        this.Sig = Sig_;
        this.Ret = this.Sig.Ret;
        this.Arg0 = this.Sig.Arg0;
        this.Arg1 = this.Sig.Arg1;
        this.Arg2 = this.Sig.Arg2;
        this.Arg3 = this.Sig.Arg3;
    };

    glsBuiltinPrecisionTests.Func.prototype = Object.create(glsBuiltinPrecisionTests.FuncBase.prototype);
    glsBuiltinPrecisionTests.Func.prototype.constructor = glsBuiltinPrecisionTests.Func;

    /**
     * @param {Array<glsBuiltinPrecisionTests.ExprBase>} args
     * @return {string}
     */
    glsBuiltinPrecisionTests.Func.prototype.print = function(args) {
        return this.doPrint(args);
    };

    /**
     * @param {glsBuiltinPrecisionTests.EvalContext} ctx
     * @param {glsBuiltinPrecisionTests.Intervals=} Iarg0
     * @param {glsBuiltinPrecisionTests.Intervals=} Iarg1
     * @param {glsBuiltinPrecisionTests.Intervals=} Iarg2
     * @param {glsBuiltinPrecisionTests.Intervals=} Iarg3
     * @return {glsBuiltinPrecisionTests.Intervals}
     */
    glsBuiltinPrecisionTests.Func.prototype.applyFunction = function(ctx, Iarg0, Iarg1, Iarg2, Iarg3, variablenames) {
        return this.applyArgs(ctx, new glsBuiltinPrecisionTests.Tuple4(Iarg0, Iarg1, Iarg2, Iarg3), variablenames);
    };

    /**
     * @param {glsBuiltinPrecisionTests.EvalContext} ctx
     * @param {glsBuiltinPrecisionTests.Tuple4} args
     * @return {glsBuiltinPrecisionTests.Intervals}
     */
    glsBuiltinPrecisionTests.Func.prototype.applyArgs = function(ctx, args, variablenames) {
        return this.doApply(ctx, args, variablenames);
    };

    /**
     * @return {glsBuiltinPrecisionTests.ParamNames}
     */
    glsBuiltinPrecisionTests.Func.prototype.getParamNames = function() {
        return this.doGetParamNames();
    };

    /**
     * @param {Array<glsBuiltinPrecisionTests.ExprBase>} args
     * @return {string}
     */
    glsBuiltinPrecisionTests.Func.prototype.doPrint = function(args) {
        /** type{string} */ var os = this.getName() + '(';

        // TODO: fix the generics
        for (var i = 0; i < args.length; i++)
            if (glsBuiltinPrecisionTests.isTypeValid(args[i].typename)) {
                if (i != 0)
                    os += ', ';
                os += args[i];
            }

        os += ')';

        return os;
    };

    /**
     * @return {glsBuiltinPrecisionTests.ParamNames} args
     */
    glsBuiltinPrecisionTests.Func.prototype.doGetParamNames = function() {
        /** @type {glsBuiltinPrecisionTests.ParamNames} */ var names = new glsBuiltinPrecisionTests.Tuple4('a', 'b', 'c', 'd');
        return names;
    };

    /**
     * @constructor
     * @extends {glsBuiltinPrecisionTests.Func}
     * @param {glsBuiltinPrecisionTests.Signature} Sig template <typename Sig>
     *
     */
    glsBuiltinPrecisionTests.PrimitiveFunc = function(Sig) {
        glsBuiltinPrecisionTests.Func.call(this, Sig);
        this.Ret = Sig.Ret;
    };

    glsBuiltinPrecisionTests.PrimitiveFunc.prototype = Object.create(glsBuiltinPrecisionTests.Func.prototype);
    glsBuiltinPrecisionTests.PrimitiveFunc.prototype.constructor = glsBuiltinPrecisionTests.PrimitiveFunc;

    /**
     * @constructor
     * @extends {glsBuiltinPrecisionTests.PrimitiveFunc}
     * @param {string} typename
     *
     */
    glsBuiltinPrecisionTests.Cond = function(typename) {
        var sig = new glsBuiltinPrecisionTests.Signature(typename, 'boolean', typename, typename);
        glsBuiltinPrecisionTests.PrimitiveFunc.call(this, sig);
    };

    setParentClass(glsBuiltinPrecisionTests.Cond, glsBuiltinPrecisionTests.PrimitiveFunc);

    glsBuiltinPrecisionTests.Cond.prototype.getName = function() {
        return '_cond';
    };

    glsBuiltinPrecisionTests.Cond.prototype.doPrint = function(args) {
        var str = '(' + args[0] + ' ? ' + args[1] + ' : ' + args[2] + ')';
        return str;
    };

    glsBuiltinPrecisionTests.Cond.prototype.doApply = function(ctx, iargs) {
        var ret;
        if (glsBuiltinPrecisionTests.contains(this.Sig.Arg0, iargs.a, 1))
            ret = iargs.b;
        if (glsBuiltinPrecisionTests.contains(this.Sig.Arg0, iargs.a, 0)) {
            if (ret)
                ret = glsBuiltinPrecisionTests.union(this.Sig.Ret, ret, iargs.c);
            else
                ret = iargs.c;
        }
        if (ret)
            return ret;
        return new tcuInterval.Interval();
    };

    /**
     * If multipleInputs is false, GenVec duplicates first input to proper size
     * @constructor
     * @extends {glsBuiltinPrecisionTests.PrimitiveFunc}
     * @param {number} size
     * @param {boolean=} multipleInputs
     */
    glsBuiltinPrecisionTests.GenVec = function(size, multipleInputs) {
        var vecName = glsBuiltinPrecisionTests.sizeToName(size);
        var p = [
            size >= 1 ? 'float' : undefined,
            size >= 2 ? 'float' : undefined,
            size >= 3 ? 'float' : undefined,
            size >= 4 ? 'float' : undefined
        ];
        var sig = new glsBuiltinPrecisionTests.Signature(vecName, p[0], p[1], p[2], p[3]);
        glsBuiltinPrecisionTests.PrimitiveFunc.call(this, sig);
        this.size = size;
        this.vecName = vecName;
        this.multipleInputs = multipleInputs || false;
    };

    setParentClass(glsBuiltinPrecisionTests.GenVec, glsBuiltinPrecisionTests.PrimitiveFunc);

    glsBuiltinPrecisionTests.GenVec.prototype.getName = function() {
        return this.vecName;
    };

    glsBuiltinPrecisionTests.GenVec.prototype.doApply = function(ctx, iargs) {
        if (this.size == 1)
            return iargs.a;

        var ret = this.multipleInputs ?
                        [iargs.a, iargs.b, iargs.c, iargs.d] :
                        [iargs.a, iargs.a, iargs.a, iargs.a];

        return ret.slice(0, this.size);
    };

    /**
     * @constructor
     * @extends {glsBuiltinPrecisionTests.PrimitiveFunc}
     * @param {number} rows
     * @param {number} cols
     */
    glsBuiltinPrecisionTests.GenMat = function(rows, cols) {
        var name = glsBuiltinPrecisionTests.dataTypeNameOfMatrix('float', rows, cols);
        var vecName = glsBuiltinPrecisionTests.sizeToName(rows);
        var p = [
            cols >= 1 ? vecName : undefined,
            cols >= 2 ? vecName : undefined,
            cols >= 3 ? vecName : undefined,
            cols >= 4 ? vecName : undefined
        ];
        var sig = new glsBuiltinPrecisionTests.Signature(name, p[0], p[1], p[2], p[3]);
        glsBuiltinPrecisionTests.PrimitiveFunc.call(this, sig);
        this.rows = rows;
        this.cols = cols;
        this.name = name;
        this.vecName = vecName;
    };

    setParentClass(glsBuiltinPrecisionTests.GenMat, glsBuiltinPrecisionTests.PrimitiveFunc);

    glsBuiltinPrecisionTests.GenMat.prototype.getName = function() {
        return this.name;
    };

    glsBuiltinPrecisionTests.GenMat.prototype.doApply = function(ctx, iargs) {
        var ret = new tcuMatrix.Matrix(this.rows, this.cols);
        var inputs = [iargs.a, iargs.b, iargs.c, iargs.d];

        for (var i = 0; i < this.rows; i++)
            for (var j = 0; j < this.cols; j++)
                ret.set(i, j, inputs[j][i]);
        return ret;
    };

    /**
     * @constructor
     * @extends {glsBuiltinPrecisionTests.PrimitiveFunc}
     * @param {string} typename
     *
     */
    glsBuiltinPrecisionTests.CompareOperator = function(typename) {
        var sig = new glsBuiltinPrecisionTests.Signature('boolean', typename, typename);
        glsBuiltinPrecisionTests.PrimitiveFunc.call(this, sig);
    };

    setParentClass(glsBuiltinPrecisionTests.CompareOperator, glsBuiltinPrecisionTests.PrimitiveFunc);

    glsBuiltinPrecisionTests.CompareOperator.prototype.doPrint = function(args) {
        var str = '(' + args[0] + this.getSymbol() + args[1] + ')';
        return str;
    };

    glsBuiltinPrecisionTests.CompareOperator.prototype.doApply = function(ctx, iargs) {
        var arg0 = iargs.a;
        var arg1 = iargs.b;

        var ret = new tcuInterval.Interval();

        if (this.canSucceed(arg0, arg1))
            ret = new tcuInterval.Interval(1);
        if (this.canFail(arg0, arg1))
            ret.operatorOrAssignBinary(new tcuInterval.Interval(0));

        return ret;
    };

    /**
     * @return {string}
     */
    glsBuiltinPrecisionTests.CompareOperator.prototype.getSymbol = function() {
        throw new Error('Virtual function. Please override.');
    };

    /**
     * @param {tcuInterval.Interval} arg0
     * @param {tcuInterval.Interval} arg1
     * @return {boolean}
     */
    glsBuiltinPrecisionTests.CompareOperator.prototype.canSucceed = function(arg0, arg1) {
        throw new Error('Virtual function. Please override.');
    };
    /**
     * @param {tcuInterval.Interval} arg0
     * @param {tcuInterval.Interval} arg1
     * @return {boolean}
     */
    glsBuiltinPrecisionTests.CompareOperator.prototype.canFail = function(arg0, arg1) {
        throw new Error('Virtual function. Please override.');
    };

    /**
     * @constructor
     * @extends {glsBuiltinPrecisionTests.CompareOperator}
     * @param {string} typename
     *
     */
    glsBuiltinPrecisionTests.LessThan = function(typename) {
        glsBuiltinPrecisionTests.CompareOperator.call(this, typename);
    };

    setParentClass(glsBuiltinPrecisionTests.LessThan, glsBuiltinPrecisionTests.CompareOperator);

    glsBuiltinPrecisionTests.LessThan.prototype.getSymbol = function() {
        return '<';
    };

    glsBuiltinPrecisionTests.LessThan.prototype.canSucceed = function(a, b) {
       return (a.lo() < b.hi());
    };

    glsBuiltinPrecisionTests.LessThan.prototype.canFail = function(a, b) {
        return !(a.hi() < b.lo());
    };

    /**
     * @constructor
     * @extends {glsBuiltinPrecisionTests.PrimitiveFunc}
     *
     */
    glsBuiltinPrecisionTests.FloatFunc1 = function() {
        var sig = new glsBuiltinPrecisionTests.Signature('float', 'float');
        glsBuiltinPrecisionTests.PrimitiveFunc.call(this, sig);
    };

    glsBuiltinPrecisionTests.FloatFunc1.prototype = Object.create(glsBuiltinPrecisionTests.PrimitiveFunc.prototype);
    glsBuiltinPrecisionTests.FloatFunc1.prototype.constructor = glsBuiltinPrecisionTests.FloatFunc1;

    /**
     * @param {glsBuiltinPrecisionTests.EvalContext} ctx
     * @param {glsBuiltinPrecisionTests.Tuple4} iargs
     * @return {tcuInterval.Interval}
     */
    glsBuiltinPrecisionTests.FloatFunc1.prototype.doApply = function(ctx, iargs) {
        var a = /** @type {tcuInterval.Interval} */ (iargs.a);
        return this.applyMonotone(ctx, a);
    };

    /**
     * @param {glsBuiltinPrecisionTests.EvalContext} ctx
     * @param {tcuInterval.Interval} iarg0
     * @return {tcuInterval.Interval}
     */
    glsBuiltinPrecisionTests.FloatFunc1.prototype.applyMonotone = function(ctx, iarg0) {
        /** @type {tcuInterval.Interval} */ var ret = new tcuInterval.Interval();

        /**
         * @param {number=} x
         * @param {number=} y
         * @return {tcuInterval.Interval}
         */
        var body = function(x, y) {
            x = x || 0;
            return this.applyPoint(ctx, x);
        };
        ret = tcuInterval.applyMonotone1(iarg0, body.bind(this));

        ret.operatorOrAssignBinary(this.innerExtrema(ctx, iarg0));

        ret.operatorAndAssignBinary(this.getCodomain().operatorOrBinary(new tcuInterval.Interval(NaN)));

        return ctx.format.convert(ret);
    };

    /**
     * @param {glsBuiltinPrecisionTests.EvalContext} ctx
     * @param {tcuInterval.Interval} iargs
     * @return {tcuInterval.Interval}
     */
    glsBuiltinPrecisionTests.FloatFunc1.prototype.innerExtrema = function(ctx, iargs) {
        return new tcuInterval.Interval(); // empty interval, i.e. no extrema
    };

    /**
     * @param {glsBuiltinPrecisionTests.EvalContext} ctx
     * @param {number} arg0
     * @return {tcuInterval.Interval}
     */
    glsBuiltinPrecisionTests.FloatFunc1.prototype.applyPoint = function(ctx, arg0) {
        var exact = this.applyExact(arg0);
        var prec = this.precision(ctx, exact, arg0);

        var a = new tcuInterval.Interval(exact);
        var b = tcuInterval.withNumbers(-prec, prec);
        return tcuInterval.Interval.operatorSum(a, b);
    };

    /**
     * @param {number} x
     * @return {number}
     */
    glsBuiltinPrecisionTests.FloatFunc1.prototype.applyExact = function(x) {
        throw new Error('Internal error. Cannot apply');
    };

    /**
     * @return {tcuInterval.Interval}
     */
    glsBuiltinPrecisionTests.FloatFunc1.prototype.getCodomain = function() {
        return tcuInterval.unbounded(true);
    };

    /**
     * @param {glsBuiltinPrecisionTests.EvalContext} ctx
     * @param {number} x
     * @param {number} y
     * @return {number}
     */
    glsBuiltinPrecisionTests.FloatFunc1.prototype.precision = function(ctx, x, y) {
        return 0;
    };

    /**
     * @constructor
     * @extends {glsBuiltinPrecisionTests.FloatFunc1}
     */
    glsBuiltinPrecisionTests.Negate = function() {
        glsBuiltinPrecisionTests.FloatFunc1.call(this);
    };

    setParentClass(glsBuiltinPrecisionTests.Negate, glsBuiltinPrecisionTests.FloatFunc1);

    glsBuiltinPrecisionTests.Negate.prototype.getName = function() {
        return '_negate';
    };

    glsBuiltinPrecisionTests.Negate.prototype.doPrint = function(args) {
        return '-' + args[0];
    };

    glsBuiltinPrecisionTests.Negate.prototype.precision = function(ctx, ret, x) {
        return 0;
    };
    glsBuiltinPrecisionTests.Negate.prototype.applyExact = function(x) {
        return -x;
    };

    /**
     * @constructor
     * @extends {glsBuiltinPrecisionTests.FloatFunc1}
     */
    glsBuiltinPrecisionTests.InverseSqrt = function() {
        glsBuiltinPrecisionTests.FloatFunc1.call(this);
    };

    setParentClass(glsBuiltinPrecisionTests.InverseSqrt, glsBuiltinPrecisionTests.FloatFunc1);

    glsBuiltinPrecisionTests.InverseSqrt.prototype.getName = function() {
        return 'inversesqrt';
    };

    glsBuiltinPrecisionTests.InverseSqrt.prototype.precision = function(ctx, ret, x) {
        if (x <= 0)
            return NaN;
        return ctx.format.ulp(ret, 2.0);
    };

    glsBuiltinPrecisionTests.InverseSqrt.prototype.applyExact = function(x) {
        return 1 / Math.sqrt(x);
    };

    glsBuiltinPrecisionTests.InverseSqrt.prototype.getCodomain = function() {
        return tcuInterval.withNumbers(0, Infinity);
    };

    /**
     * @constructor
     * @extends {glsBuiltinPrecisionTests.FloatFunc1}
     */
    glsBuiltinPrecisionTests.Round = function() {
        glsBuiltinPrecisionTests.FloatFunc1.call(this);
    };

    setParentClass(glsBuiltinPrecisionTests.Round, glsBuiltinPrecisionTests.FloatFunc1);

    glsBuiltinPrecisionTests.Round.prototype.getName = function() {
        return 'round';
    };

    glsBuiltinPrecisionTests.Round.prototype.precision = function(ctx, ret, x) {
        return 0;
    };

    glsBuiltinPrecisionTests.Round.prototype.applyPoint = function(ctx, x) {
        var truncated = Math.trunc(x);
        var fract = x - truncated;
        var ret = new tcuInterval.Interval();

        // When x is inf or -inf, truncated would be inf or -inf too. Then fract
        // would be NaN (inf - inf). While in native c code, it would be 0 (inf) or -0 (-inf).
        // This behavior in JS differs from that in native c code.
        if (Math.abs(fract) <= 0.5 || isNaN(fract))
            ret.operatorOrAssignBinary(new tcuInterval.Interval(truncated));
        if (Math.abs(fract) >= 0.5)
            ret.operatorOrAssignBinary(new tcuInterval.Interval(truncated + deMath.deSign(fract)));

        return ret;
    };

    /**
     * @constructor
     * @extends {glsBuiltinPrecisionTests.FloatFunc1}
     * @param {string} name
     * @param {tcuInterval.DoubleFunc1} func
     */
    glsBuiltinPrecisionTests.CFloatFunc1 = function(name, func) {
        glsBuiltinPrecisionTests.FloatFunc1.call(this);
        /** @type {string} */ this.m_name = name;
        /** @type {tcuInterval.DoubleFunc1} */this.m_func = func;
    };

    glsBuiltinPrecisionTests.CFloatFunc1.prototype = Object.create(glsBuiltinPrecisionTests.FloatFunc1.prototype);
    glsBuiltinPrecisionTests.CFloatFunc1.prototype.constructor = glsBuiltinPrecisionTests.CFloatFunc1;

    /**
     * @return {string}
     */
    glsBuiltinPrecisionTests.CFloatFunc1.prototype.getName = function() {
        return this.m_name;
    };

    /**
     * @param {number} x
     * @return {number}
     */
    glsBuiltinPrecisionTests.CFloatFunc1.prototype.applyExact = function(x) {
        return this.m_func(x);
    };

    /**
     * PrimitiveFunc<Signature<float, float, float> >
     * @constructor
     * @extends {glsBuiltinPrecisionTests.PrimitiveFunc}
     */
    glsBuiltinPrecisionTests.FloatFunc2 = function() {
        /** @type {glsBuiltinPrecisionTests.Signature} */ var Sig = new glsBuiltinPrecisionTests.Signature('float', 'float', 'float');
        glsBuiltinPrecisionTests.PrimitiveFunc.call(this, Sig);
    };

    glsBuiltinPrecisionTests.FloatFunc2.prototype = Object.create(glsBuiltinPrecisionTests.PrimitiveFunc.prototype);
    glsBuiltinPrecisionTests.FloatFunc2.prototype.constructor = glsBuiltinPrecisionTests.FloatFunc2;

    /**
     * @param {glsBuiltinPrecisionTests.EvalContext} ctx
     * @param {glsBuiltinPrecisionTests.Tuple4} iargs
     * @return {tcuInterval.Interval}
     */
    glsBuiltinPrecisionTests.FloatFunc2.prototype.doApply = function(ctx, iargs) {
        var a = /** @type {tcuInterval.Interval} */ (iargs.a);
        var b = /** @type {tcuInterval.Interval} */ (iargs.b);
        return this.applyMonotone(ctx, a, b);
    };

    /**
     * @param {glsBuiltinPrecisionTests.EvalContext} ctx
     * @param {tcuInterval.Interval} xi
     * @param {tcuInterval.Interval} yi
     * @return {tcuInterval.Interval}
     */
    glsBuiltinPrecisionTests.FloatFunc2.prototype.applyMonotone = function(ctx, xi, yi) {
        /** @type {tcuInterval.Interval} */ var ret = new tcuInterval.Interval();

        /**
         * @param {number=} x
         * @param {number=} y
         * @return {tcuInterval.Interval}
         */
        var body = function(x, y) {
            x = x || 0;
            y = y || 0;
            return this.applyPoint(ctx, x, y);
        };
        ret = tcuInterval.applyMonotone2(xi, yi, body.bind(this));

        ret.operatorOrAssignBinary(this.innerExtrema(ctx, xi, yi));

        ret.operatorAndAssignBinary(this.getCodomain().operatorOrBinary(new tcuInterval.Interval(NaN)));

        return ctx.format.convert(ret);
    };

    /**
     * @param {glsBuiltinPrecisionTests.EvalContext} ctx
     * @param {tcuInterval.Interval} xi
     * @param {tcuInterval.Interval} yi
     * @return {tcuInterval.Interval}
     */
    glsBuiltinPrecisionTests.FloatFunc2.prototype.innerExtrema = function(ctx, xi, yi) {
        return new tcuInterval.Interval(); // empty interval, i.e. no extrema
    };

    /**
     * @param {glsBuiltinPrecisionTests.EvalContext} ctx
     * @param {number} x
     * @param {number} y
     * @return {tcuInterval.Interval}
     */
    glsBuiltinPrecisionTests.FloatFunc2.prototype.applyPoint = function(ctx, x, y) {
        /** @type {number} */ var exact = this.applyExact(x, y);
        var prec = this.precision(ctx, exact, x, y);

        var a = new tcuInterval.Interval(exact);
        var b = tcuInterval.withNumbers(-prec, prec);
        return tcuInterval.Interval.operatorSum(a, b);
    };

    /**
     * @param {number} x
     * @param {number} y
     * @return {number}
     */
    glsBuiltinPrecisionTests.FloatFunc2.prototype.applyExact = function(x, y) {
        throw new Error('Virtual function. Please override');
    };

    /**
     * @return {tcuInterval.Interval}
     */
    glsBuiltinPrecisionTests.FloatFunc2.prototype.getCodomain = function() {
        return tcuInterval.unbounded(true);
    };

    /**
     * @param {glsBuiltinPrecisionTests.EvalContext} ctx
     * @param {number} ret
     * @param {number} x
     * @param {number} y
     * @return {number}
     */
    glsBuiltinPrecisionTests.FloatFunc2.prototype.precision = function(ctx, ret, x, y) {
        throw new Error('Virtual function. Please override');
    };

    /**
     * @constructor
     * @extends {glsBuiltinPrecisionTests.FloatFunc2}
     * @param {string} name
     * @param {tcuInterval.DoubleFunc2} func
     */
    glsBuiltinPrecisionTests.CFloatFunc2 = function(name, func) {
        glsBuiltinPrecisionTests.FloatFunc2.call(this);
        /** @type {string} */ this.m_name = name;
        /** @type {tcuInterval.DoubleFunc2} */ this.m_func = func;
    };

    glsBuiltinPrecisionTests.CFloatFunc2.prototype = Object.create(glsBuiltinPrecisionTests.FloatFunc2.prototype);
    glsBuiltinPrecisionTests.CFloatFunc2.prototype.constructor = glsBuiltinPrecisionTests.CFloatFunc2;

    /**
     * @return {string}
     */
    glsBuiltinPrecisionTests.CFloatFunc2.prototype.getName = function() {
        return this.m_name;
    };

    /**
     * @param {number} x
     * @param {number} y
     * @return {number}
     */
    glsBuiltinPrecisionTests.CFloatFunc2.prototype.applyExact = function(x, y) {
        return this.m_func(x, y);
    };

    /**
     * @constructor
     * @extends {glsBuiltinPrecisionTests.FloatFunc2}
     */
    glsBuiltinPrecisionTests.InfixOperator = function() {
        glsBuiltinPrecisionTests.FloatFunc2.call(this);
    };

    glsBuiltinPrecisionTests.InfixOperator.prototype = Object.create(glsBuiltinPrecisionTests.FloatFunc2.prototype);
    glsBuiltinPrecisionTests.InfixOperator.prototype.constructor = glsBuiltinPrecisionTests.InfixOperator;

    /**
     * @return {string}
     */
    glsBuiltinPrecisionTests.InfixOperator.prototype.getSymbol = function() {
        glsBuiltinPrecisionTests.FloatFunc2.call(this);
        return '';
    };

    /**
     * @param {Array<glsBuiltinPrecisionTests.ExprBase>} args
     * @return {string}
     */
    glsBuiltinPrecisionTests.InfixOperator.prototype.doPrint = function(args) {
        return '(' + args[0] + ' ' + this.getSymbol() + ' ' + args[1] + ')';
    };

    /**
     * @param {glsBuiltinPrecisionTests.EvalContext} ctx
     * @param {number} x
     * @param {number} y
     * @return {tcuInterval.Interval}
     */
    glsBuiltinPrecisionTests.InfixOperator.prototype.applyPoint = function(ctx, x, y) {
        /** @type {number} */ var exact = this.applyExact(x, y);

        // Allow either representable number on both sides of the exact value,
        // but require exactly representable values to be preserved.
        return ctx.format.roundOut(new tcuInterval.Interval(exact), isFinite(x) && isFinite(y));
    };

    /**
     * @param {glsBuiltinPrecisionTests.EvalContext} ctx
     * @param {number} x
     * @param {number} y
     * @param {number} z
     * @return {number}
     */
    glsBuiltinPrecisionTests.InfixOperator.prototype.precision = function(ctx, x, y, z) {
        return 0;
    };

    /**
     * Signature<float, float, float, float>
     * @constructor
     * @extends {glsBuiltinPrecisionTests.PrimitiveFunc}
     */
    glsBuiltinPrecisionTests.FloatFunc3 = function() {
        /** @type {glsBuiltinPrecisionTests.Signature} */ var sig = new glsBuiltinPrecisionTests.Signature('float', 'float', 'float', 'float');
        glsBuiltinPrecisionTests.PrimitiveFunc.call(this, sig);
    };

    glsBuiltinPrecisionTests.FloatFunc3.prototype = Object.create(glsBuiltinPrecisionTests.PrimitiveFunc.prototype);
    glsBuiltinPrecisionTests.FloatFunc3.prototype.constructor = glsBuiltinPrecisionTests.FloatFunc3;

    /**
     * @param {glsBuiltinPrecisionTests.EvalContext} ctx
     * @param {glsBuiltinPrecisionTests.Tuple4} iargs
     * @return {tcuInterval.Interval}
     */
    glsBuiltinPrecisionTests.FloatFunc3.prototype.doApply = function(ctx, iargs) {
        var a = /** @type {tcuInterval.Interval} */ (iargs.a);
        var b = /** @type {tcuInterval.Interval} */ (iargs.b);
        var c = /** @type {tcuInterval.Interval} */ (iargs.c);
        var retVal = this.applyMonotone(ctx, a, b, c);
        return retVal;
    };

    /**
     * @param {glsBuiltinPrecisionTests.EvalContext} ctx
     * @param {tcuInterval.Interval} xi
     * @param {tcuInterval.Interval} yi
     * @param {tcuInterval.Interval} zi
     * @return {tcuInterval.Interval}
     */
    glsBuiltinPrecisionTests.FloatFunc3.prototype.applyMonotone = function(ctx, xi, yi, zi) {
        /**
         * @param {number=} x
         * @param {number=} y
         * @param {number=} z
         * @return {tcuInterval.Interval}
         */
        var body = function(x, y, z) {
            x = x || 0;
            y = y || 0;
            z = z || 0;
            return this.applyPoint(ctx, x, y, z);
        };
        var ret = tcuInterval.applyMonotone3(xi, yi, zi, body.bind(this));
        var retVal;

        ret.operatorOrAssignBinary(this.innerExtrema(ctx, xi, yi, zi));

        ret.operatorAndAssignBinary(this.getCodomain().operatorOrBinary(new tcuInterval.Interval(NaN)));

        retVal = ctx.format.convert(ret);
        return retVal;
    };

    /**
     * @param {glsBuiltinPrecisionTests.EvalContext} ctx
     * @param {tcuInterval.Interval} xi
     * @param {tcuInterval.Interval} yi
     * @param {tcuInterval.Interval} zi
     * @return {tcuInterval.Interval}
     */
    glsBuiltinPrecisionTests.FloatFunc3.prototype.innerExtrema = function(ctx, xi, yi, zi) {
        return new tcuInterval.Interval(); // empty interval, i.e. no extrema
    };

    /**
     * @param {glsBuiltinPrecisionTests.EvalContext} ctx
     * @param {number} x
     * @param {number} y
     * @param {number} z
     * @return {tcuInterval.Interval}
     */
    glsBuiltinPrecisionTests.FloatFunc3.prototype.applyPoint = function(ctx, x, y, z) {
        /** @type {number} */ var exact = this.applyExact(x, y, z);
        /** @type {number} */ var prec = this.precision(ctx, exact, x, y, z);

        var a = new tcuInterval.Interval(exact);
        var b = tcuInterval.withNumbers(-prec, prec);
        return tcuInterval.Interval.operatorSum(a, b);
    };

    /**
     * @param {number} x
     * @param {number} y
     * @param {number} z
     * @return {number}
     */
    glsBuiltinPrecisionTests.FloatFunc3.prototype.applyExact = function(x, y, z) {
        throw new Error('Virtual function. Please override');
    };

    /**
     * @param {glsBuiltinPrecisionTests.EvalContext} ctx
     * @param {number} result
     * @param {number} x
     * @param {number} y
     * @param {number} z
     * @return {number}
     */
    glsBuiltinPrecisionTests.FloatFunc3.prototype.precision = function(ctx, result, x, y, z) {
        throw new Error('Virtual function. Please override');
    };

    /**
     * @return {tcuInterval.Interval}
     */
    glsBuiltinPrecisionTests.FloatFunc3.prototype.getCodomain = function() {
        return tcuInterval.unbounded(true);
    };

    /**
     * @constructor
     * @extends {glsBuiltinPrecisionTests.FloatFunc3}
     */
    glsBuiltinPrecisionTests.Clamp = function() {
        glsBuiltinPrecisionTests.FloatFunc3.call(this);
    };

    setParentClass(glsBuiltinPrecisionTests.Clamp, glsBuiltinPrecisionTests.FloatFunc3);

    glsBuiltinPrecisionTests.Clamp.prototype.getName = function() {
        return 'clamp';
    };

    glsBuiltinPrecisionTests.Clamp.prototype.applyExact = function(x, minVal, maxVal) {
        var debug = false;
        var retVal;

        retVal = deMath.clamp(x, minVal, maxVal);
        if (debug) {
            console.log('> minVal: ' + minVal);
            console.log('> maxVal: ' + maxVal);
            console.log('> x: ' + x);
            console.log('> ret: ' + retVal);
        }
        return retVal;

    };

    glsBuiltinPrecisionTests.Clamp.prototype.precision = function(ctx, result, x, minVal, maxVal) {
        var debug = false;
        var retVal;

        retVal = minVal > maxVal ? NaN : 0;

        if (debug) {
            console.log('precision> minVal: ' + minVal);
            console.log('precision> maxVal: ' + maxVal);
            console.log('precision> x: ' + x);
            console.log('precision> ret: ' + retVal);
        }

        return retVal;
    };

    /**
     * @constructor
     * @extends {glsBuiltinPrecisionTests.InfixOperator}
     */
    glsBuiltinPrecisionTests.Add = function() {
        glsBuiltinPrecisionTests.InfixOperator.call(this);
    };

    glsBuiltinPrecisionTests.Add.prototype = Object.create(glsBuiltinPrecisionTests.InfixOperator.prototype);
    glsBuiltinPrecisionTests.Add.prototype.constructor = glsBuiltinPrecisionTests.Add;

    /**
     * @return {string}
     */
    glsBuiltinPrecisionTests.Add.prototype.getName = function() {
        return 'add';
    };

    /**
     * @return {string}
     */
    glsBuiltinPrecisionTests.Add.prototype.getSymbol = function() {
        return '+';
    };

    /**
     * @param {glsBuiltinPrecisionTests.EvalContext} ctx
     * @param {glsBuiltinPrecisionTests.Tuple4} iargs
     * @return {tcuInterval.Interval}
     */
    glsBuiltinPrecisionTests.Add.prototype.doApply = function(ctx, iargs) {
        var a = /** @type {tcuInterval.Interval} */ (iargs.a);
        var b = /** @type {tcuInterval.Interval} */ (iargs.b);
        // Fast-path for common case
        if (iargs.a.isOrdinary() && iargs.b.isOrdinary()) {
            /** type{tcuInterval.Interval} */ var ret;
            ret = tcuInterval.setIntervalBounds(
                function(dummy) {
                    return iargs.a.lo() + iargs.b.lo();
                },
                function(dummy) {
                    return iargs.a.hi() + iargs.b.hi();
                });
            return ctx.format.convert(ctx.format.roundOut(ret, true));
        }
        return this.applyMonotone(ctx, a, b);
    };

    /**
     * @param {number} x
     * @param {number} y
     * @return {number}
     */
    glsBuiltinPrecisionTests.Add.prototype.applyExact = function(x, y) {
        return x + y;
    };

    /**
     * @constructor
     * @extends {glsBuiltinPrecisionTests.InfixOperator}
     */
    glsBuiltinPrecisionTests.Sub = function() {
        glsBuiltinPrecisionTests.InfixOperator.call(this);
    };

    glsBuiltinPrecisionTests.Sub.prototype = Object.create(glsBuiltinPrecisionTests.InfixOperator.prototype);
    glsBuiltinPrecisionTests.Sub.prototype.constructor = glsBuiltinPrecisionTests.Sub;

    /**
     * @return {string}
     */
    glsBuiltinPrecisionTests.Sub.prototype.getName = function() {
        return 'sub';
    };

    /**
     * @return {string}
     */
    glsBuiltinPrecisionTests.Sub.prototype.getSymbol = function() {
        return '-';
    };

    /**
     * @param {glsBuiltinPrecisionTests.EvalContext} ctx
     * @param {glsBuiltinPrecisionTests.Tuple4} iargs
     * @return {tcuInterval.Interval}
     */
    glsBuiltinPrecisionTests.Sub.prototype.doApply = function(ctx, iargs) {
        var a = /** @type {tcuInterval.Interval} */ (iargs.a);
        var b = /** @type {tcuInterval.Interval} */ (iargs.b);
        var retVal;

        // Fast-path for common case
        if (iargs.a.isOrdinary() && iargs.b.isOrdinary()) {
            /** type{tcuInterval.Interval} */ var ret;
            ret = tcuInterval.setIntervalBounds(
                function(dummy) {
                    return iargs.a.lo() - iargs.b.hi();
                },
                function(dummy) {
                    return iargs.a.hi() - iargs.b.lo();
                });
            return ctx.format.convert(ctx.format.roundOut(ret, true));
        }
        retVal = this.applyMonotone(ctx, a, b);
        return retVal;
    };

    /**
     * @param {number} x
     * @param {number} y
     * @return {number}
     */
    glsBuiltinPrecisionTests.Sub.prototype.applyExact = function(x, y) {
        return x - y;
    };

    /**
     * @constructor
     * @extends {glsBuiltinPrecisionTests.InfixOperator}
     */
    glsBuiltinPrecisionTests.Mul = function() {
        glsBuiltinPrecisionTests.InfixOperator.call(this);
    };

    glsBuiltinPrecisionTests.Mul.prototype = Object.create(glsBuiltinPrecisionTests.InfixOperator.prototype);
    glsBuiltinPrecisionTests.Mul.prototype.constructor = glsBuiltinPrecisionTests.Mul;

    /**
     * @return {string}
     */
    glsBuiltinPrecisionTests.Mul.prototype.getName = function() {
        return 'mul';
    };

    /**
     * @return {string}
     */
    glsBuiltinPrecisionTests.Mul.prototype.getSymbol = function() {
        return '*';
    };

    glsBuiltinPrecisionTests.isNegative = function(n) {
        return ((n = +n) || 1 / n) < 0;
    };

   /**
     * @param {glsBuiltinPrecisionTests.EvalContext} ctx
     * @param {glsBuiltinPrecisionTests.Tuple4} iargs
     * @return {tcuInterval.Interval}
     */
    glsBuiltinPrecisionTests.Mul.prototype.doApply = function(ctx, iargs) {
        var a = /** @type {tcuInterval.Interval} */ (iargs.a);
        var b = /** @type {tcuInterval.Interval} */ (iargs.b);
        // Fast-path for common case
        if (iargs.a.isOrdinary() && iargs.b.isOrdinary()) {
            /** type{tcuInterval.Interval} */ var ret = new tcuInterval.Interval();
            if (glsBuiltinPrecisionTests.isNegative(a.hi())) {
                a = a.operatorNegative();
                b = b.operatorNegative();
            }
            if (a.lo() >= 0 && b.lo() >= 0) {
                ret = tcuInterval.setIntervalBounds(
                    function(dummy) {
                        return iargs.a.lo() * iargs.b.lo();
                    },
                    function(dummy) {
                        return iargs.a.hi() * iargs.b.hi();
                    });
                return ctx.format.convert(ctx.format.roundOut(ret, true));
            }
            if (a.lo() >= 0 && b.hi() <= 0) {
                ret = tcuInterval.setIntervalBounds(
                    function(dummy) {
                        return iargs.a.hi() * iargs.b.lo();
                    },
                    function(dummy) {
                        return iargs.a.lo() * iargs.b.hi();
                    });
                return ctx.format.convert(ctx.format.roundOut(ret, true));
            }
        }

        return this.applyMonotone(ctx, a, b);
    };

    /**
     * @param {number} x
     * @param {number} y
     * @return {number}
     */
    glsBuiltinPrecisionTests.Mul.prototype.applyExact = function(x, y) {
        return x * y;
    };

    /**
     * @param {glsBuiltinPrecisionTests.EvalContext} ctx
     * @param {tcuInterval.Interval} xi
     * @param {tcuInterval.Interval} yi
     * @return {tcuInterval.Interval}
     */
    glsBuiltinPrecisionTests.Mul.prototype.innerExtrema = function(ctx, xi, yi) {
        if (((xi.contains(tcuInterval.NEGATIVE_INFINITY) || xi.contains(tcuInterval.POSITIVE_INFINITY)) && yi.contains(tcuInterval.ZERO)) ||
            ((yi.contains(tcuInterval.NEGATIVE_INFINITY) || yi.contains(tcuInterval.POSITIVE_INFINITY)) && xi.contains(tcuInterval.ZERO)))
            return new tcuInterval.Interval(NaN);

        return new tcuInterval.Interval(); // empty interval, i.e. no extrema
    };

    /**
     * @constructor
     * @extends {glsBuiltinPrecisionTests.InfixOperator}
     */
    glsBuiltinPrecisionTests.Div = function() {
        glsBuiltinPrecisionTests.InfixOperator.call(this);
    };

    glsBuiltinPrecisionTests.Div.prototype = Object.create(glsBuiltinPrecisionTests.InfixOperator.prototype);
    glsBuiltinPrecisionTests.Div.prototype.constructor = glsBuiltinPrecisionTests.Div;

    /**
     * @return {string}
     */
    glsBuiltinPrecisionTests.Div.prototype.getName = function() {
        return 'div';
    };

    /**
     * @return {string}
     */
    glsBuiltinPrecisionTests.Div.prototype.getSymbol = function() {
        return '/';
    };

    /**
     * @param {glsBuiltinPrecisionTests.EvalContext} ctx
     * @param {tcuInterval.Interval} nom
     * @param {tcuInterval.Interval} den
     * @return {tcuInterval.Interval}
     */
    glsBuiltinPrecisionTests.Div.prototype.innerExtrema = function(ctx, nom, den) {
        var ret = new tcuInterval.Interval();
        if (den.contains(tcuInterval.ZERO)) {
            if (nom.contains(tcuInterval.ZERO))
                ret.operatorOrAssignBinary(tcuInterval.NAN);
            if (nom.lo() < 0 || nom.hi() > 0.0)
                ret.operatorOrAssignBinary(tcuInterval.unbounded());
        }

        return ret;
    };

    glsBuiltinPrecisionTests.Div.prototype.precision = function(ctx, ret, nom, den) {
        var fmt = ctx.format;

        // \todo [2014-03-05 lauri] Check that the limits in GLSL 3.10 are actually correct.
        // For now, we assume that division's precision is 2.5 ULP when the value is within
        // [2^MINEXP, 2^MAXEXP-1]

        if (den === 0)
            return 0; // Result must be exactly inf
        else if (deMath.deInBounds32(Math.abs(den),
                              deMath.deLdExp(1, fmt.getMinExp()),
                              deMath.deLdExp(1, fmt.getMaxExp() - 1)))
            return fmt.ulp(ret, 2.5);
        else
            return Infinity; // Can be any number, but must be a number.
    };

    /**
     * @param {number} x
     * @param {number} y
     * @return {number}
     */
    glsBuiltinPrecisionTests.Div.prototype.applyExact = function(x, y) {
        return x / y;
    };

    glsBuiltinPrecisionTests.Div.prototype.applyPoint = function(ctx, x, y) {
        var ret = glsBuiltinPrecisionTests.FloatFunc2.prototype.applyPoint.call(this, ctx, x, y);
        if (isFinite(x) && isFinite(y) && y != 0) {
            var dst = ctx.format.convert(ret);
            if (dst.contains(tcuInterval.NEGATIVE_INFINITY)) {
                ret.operatorOrAssignBinary(-ctx.format.getMaxValue());
            }
            if (dst.contains(tcuInterval.POSITIVE_INFINITY)) {
                ret.operatorOrAssignBinary(+ctx.format.getMaxValue());
            }
        }
        return ret;
    };

    /**
     * @constructor
     * @extends {glsBuiltinPrecisionTests.PrimitiveFunc}
     */
    glsBuiltinPrecisionTests.CompWiseFunc = function(typename, Sig) {
        glsBuiltinPrecisionTests.PrimitiveFunc.call(this, Sig);
        this.typename = typename;
    };

    setParentClass(glsBuiltinPrecisionTests.CompWiseFunc, glsBuiltinPrecisionTests.PrimitiveFunc);

    /**
     * @return {string}
     */
    glsBuiltinPrecisionTests.CompWiseFunc.prototype.getName = function() {
        return this.doGetScalarFunc().getName();
    };

    /**
     * @param {Array<glsBuiltinPrecisionTests.ExprBase>} args
     * @return {string}
     */
    glsBuiltinPrecisionTests.CompWiseFunc.prototype.doPrint = function(args) {
        return this.doGetScalarFunc().print(args);
    };

    /**
     * @return {glsBuiltinPrecisionTests.Func}
     */
    glsBuiltinPrecisionTests.CompWiseFunc.prototype.doGetScalarFunc = function() {
        throw new Error('Virtual function. Please override.');
    };

    /**
     * @constructor
     * @extends {glsBuiltinPrecisionTests.CompWiseFunc}
     * @param {number} rows
     * @param {number} cols
     */
    glsBuiltinPrecisionTests.CompMatFuncBase = function(rows, cols) {
        var name = glsBuiltinPrecisionTests.dataTypeNameOfMatrix('float', rows, cols);
        glsBuiltinPrecisionTests.CompWiseFunc.call(this, 'float', new glsBuiltinPrecisionTests.Signature(name, name, name));
        this.rows = rows;
        this.cols = cols;
    };

    setParentClass(glsBuiltinPrecisionTests.CompMatFuncBase, glsBuiltinPrecisionTests.CompWiseFunc);

    glsBuiltinPrecisionTests.CompMatFuncBase.prototype.doApply = function(ctx, iargs) {
        var ret = new tcuMatrix.Matrix(this.rows, this.cols);
        var fun = this.doGetScalarFunc();

        for (var row = 0; row < this.rows; ++row)
            for (var col = 0; col < this.cols; ++col)
                ret.set(row, col, fun.applyFunction(ctx,
                                                  iargs.a.get(row, col),
                                                  iargs.b.get(row, col)));

        return ret;
    };

    /**
     * @constructor
     * @extends {glsBuiltinPrecisionTests.CompMatFuncBase}
     * @param {function(new:glsBuiltinPrecisionTests.Func)} F
     * @param {number} rows
     * @param {number} cols
     */
    glsBuiltinPrecisionTests.CompMatFunc = function(F, rows, cols) {
        glsBuiltinPrecisionTests.CompMatFuncBase.call(this, rows, cols);
        this.m_function = F;
    };

    setParentClass(glsBuiltinPrecisionTests.CompMatFunc, glsBuiltinPrecisionTests.CompMatFuncBase);

    /**
     * @return {glsBuiltinPrecisionTests.Func}
     */
    glsBuiltinPrecisionTests.CompMatFunc.prototype.doGetScalarFunc = function() {
        return new this.m_function();
    };

    /**
     * @constructor
     * @extends {glsBuiltinPrecisionTests.Mul}
     */
    glsBuiltinPrecisionTests.ScalarMatrixCompMult = function() {
       glsBuiltinPrecisionTests.Mul.call(this);
    };

    setParentClass(glsBuiltinPrecisionTests.ScalarMatrixCompMult, glsBuiltinPrecisionTests.Mul);

    /**
     * @return {string}
     */
    glsBuiltinPrecisionTests.ScalarMatrixCompMult.prototype.getName = function() {
        return 'matrixCompMult';
    };

    /**
     * @param {Array<glsBuiltinPrecisionTests.ExprBase>} args
     * @return {string}
     */
    glsBuiltinPrecisionTests.ScalarMatrixCompMult.prototype.doPrint = function(args) {
        return glsBuiltinPrecisionTests.Func.prototype.doPrint.call(this, args);
    };

    /**
     * @constructor
     * @extends {glsBuiltinPrecisionTests.CompMatFunc}
     * @param {number} rows
     * @param {number} cols
     */
    glsBuiltinPrecisionTests.MatrixCompMult = function(rows, cols) {
        glsBuiltinPrecisionTests.CompMatFunc.call(this, glsBuiltinPrecisionTests.ScalarMatrixCompMult, rows, cols);
    };

    setParentClass(glsBuiltinPrecisionTests.MatrixCompMult, glsBuiltinPrecisionTests.CompMatFunc);

    /**
     * @constructor
     * @extends {glsBuiltinPrecisionTests.PrimitiveFunc}
     * @param {number} rows
     * @param {number} cols
     */
    glsBuiltinPrecisionTests.OuterProduct = function(rows, cols) {
        var name = glsBuiltinPrecisionTests.dataTypeNameOfMatrix('float', rows, cols);
        var sig = new glsBuiltinPrecisionTests.Signature(name, 'vec' + rows, 'vec' + cols);
        glsBuiltinPrecisionTests.PrimitiveFunc.call(this, sig);
        this.rows = rows;
        this.cols = cols;
    };

    setParentClass(glsBuiltinPrecisionTests.OuterProduct, glsBuiltinPrecisionTests.PrimitiveFunc);

    /**
     * @return {string}
     */
    glsBuiltinPrecisionTests.OuterProduct.prototype.getName = function() {
        return 'outerProduct';
    };

    glsBuiltinPrecisionTests.OuterProduct.prototype.doApply = function(ctx, iargs) {
        var ret = new tcuMatrix.Matrix(this.rows, this.cols);
        var mul = new glsBuiltinPrecisionTests.Mul();

        for (var row = 0; row < this.rows; ++row) {
            for (var col = 0; col < this.cols; ++col)
                ret.set(row, col, mul.applyFunction(ctx, iargs.a[row], iargs.b[col]));
        }

        return ret;
    };

    /**
     * @constructor
     * @extends {glsBuiltinPrecisionTests.PrimitiveFunc}
     * @param {number} rows
     * @param {number} cols
     */
    glsBuiltinPrecisionTests.Transpose = function(rows, cols) {
        var nameRet = glsBuiltinPrecisionTests.dataTypeNameOfMatrix('float', rows, cols);
        var nameParam = glsBuiltinPrecisionTests.dataTypeNameOfMatrix('float', cols, rows);
        var sig = new glsBuiltinPrecisionTests.Signature(nameRet, nameParam);
        glsBuiltinPrecisionTests.PrimitiveFunc.call(this, sig);
        this.rows = rows;
        this.cols = cols;
    };

    setParentClass(glsBuiltinPrecisionTests.Transpose, glsBuiltinPrecisionTests.PrimitiveFunc);

    /**
     * @return {string}
     */
    glsBuiltinPrecisionTests.Transpose.prototype.getName = function() {
        return 'transpose';
    };

    glsBuiltinPrecisionTests.Transpose.prototype.doApply = function(ctx, iargs) {
        var ret = new tcuMatrix.Matrix(this.rows, this.cols);

        for (var row = 0; row < this.rows; ++row)
            for (var col = 0; col < this.cols; ++col)
                ret.set(row, col, iargs.a.get(col, row));

        return ret;
    };

    /**
     * @constructor
     * @param {*} In
     */
    glsBuiltinPrecisionTests.Inputs = function(In) {
        // vector<typename In::In0> in0;
        // vector<typename In::In1> in1;
        // vector<typename In::In2> in2;
        // vector<typename In::In3> in3;
        this.in0 = [];
        this.in1 = [];
        this.in2 = [];
        this.in3 = [];
    };

    /**
     * @constructor
     * @param {number} size
     * @param {*} Out
     */
    glsBuiltinPrecisionTests.Outputs = function(size, Out) {
        // Outputs (size_t size) : out0(size), out1(size) {}
        this.out0 = [];
        this.out1 = [];
    };

    /**
     * @constructor
     * @param {*} In
     * @param {*} Out
     */
     glsBuiltinPrecisionTests.Variables = function(In, Out) {
        this.in0 = new glsBuiltinPrecisionTests.Variable(In.In0);
        this.in1 = new glsBuiltinPrecisionTests.Variable(In.In1);
        this.in2 = new glsBuiltinPrecisionTests.Variable(In.In2);
        this.in3 = new glsBuiltinPrecisionTests.Variable(In.In3);
        this.out0 = new glsBuiltinPrecisionTests.Variable(Out.Out0);
        this.out1 = new glsBuiltinPrecisionTests.Variable(Out.Out1);
    };

    /**
     * @constructor
     * @param {function(new:glsBuiltinPrecisionTests.Func)} F
     * @return {glsBuiltinPrecisionTests.GenFuncs}
     */
    glsBuiltinPrecisionTests.makeVectorizedFuncs = function(F) {
        return new glsBuiltinPrecisionTests.GenFuncs(
                new F(),
                new glsBuiltinPrecisionTests.VectorizedFunc(new F(), 2),
                new glsBuiltinPrecisionTests.VectorizedFunc(new F(), 3),
                new glsBuiltinPrecisionTests.VectorizedFunc(new F(), 4));
    };

    /**
     * @constructor
     * @param {glsBuiltinPrecisionTests.Typename} typename
     */
    glsBuiltinPrecisionTests.Sampling = function(typename) {
        this.typename = typename;
    };

    /**
     * @param {glsBuiltinPrecisionTests.Typename} typename
     * @param {number=} size
     * @return {glsBuiltinPrecisionTests.Sampling}
     */
    glsBuiltinPrecisionTests.SamplingFactory = function(typename, size) {
        if (size > 1)
            return new glsBuiltinPrecisionTests.DefaultSamplingVector(typename, size);
        switch (typename) {
            case 'vec4' : return new glsBuiltinPrecisionTests.DefaultSamplingVector('float', 4);
            case 'vec3' : return new glsBuiltinPrecisionTests.DefaultSamplingVector('float', 3);
            case 'vec2' : return new glsBuiltinPrecisionTests.DefaultSamplingVector('float', 2);
            case 'boolean' : return new glsBuiltinPrecisionTests.DefaultSamplingBool(typename);
            case 'float' : return new glsBuiltinPrecisionTests.DefaultSamplingFloat(typename);
            case 'mat2': return new glsBuiltinPrecisionTests.DefaultSamplingMatrix('float', 2, 2);
            case 'mat2x3': return new glsBuiltinPrecisionTests.DefaultSamplingMatrix('float', 3, 2);
            case 'mat2x4': return new glsBuiltinPrecisionTests.DefaultSamplingMatrix('float', 4, 2);
            case 'mat3x2': return new glsBuiltinPrecisionTests.DefaultSamplingMatrix('float', 2, 3);
            case 'mat3': return new glsBuiltinPrecisionTests.DefaultSamplingMatrix('float', 3, 3);
            case 'mat3x4': return new glsBuiltinPrecisionTests.DefaultSamplingMatrix('float', 4, 3);
            case 'mat4x2': return new glsBuiltinPrecisionTests.DefaultSamplingMatrix('float', 2, 4);
            case 'mat4x3': return new glsBuiltinPrecisionTests.DefaultSamplingMatrix('float', 3, 4);
            case 'mat4': return new glsBuiltinPrecisionTests.DefaultSamplingMatrix('float', 4, 4);
            case 'int' : return new glsBuiltinPrecisionTests.DefaultSamplingInt(typename);
        }
        return new glsBuiltinPrecisionTests.DefaultSamplingVoid(typename);
    };

    /**
     * @param {tcuFloatFormat.FloatFormat} fmt
     * @param {Array<*>} arr
     */
    glsBuiltinPrecisionTests.Sampling.prototype.genFixeds = function(fmt, arr) {
        throw new Error('Virtual function. Please override.');
    };

    /**
     * @param {tcuFloatFormat.FloatFormat} fmt
     * @param {gluShaderUtil.precision} precision
     * @param {deRandom.Random} random
     * @return {*}
     */
    glsBuiltinPrecisionTests.Sampling.prototype.genRandom = function(fmt, precision, random) {
        return 0;
    };

    /**
     * @return {number}
     */
    glsBuiltinPrecisionTests.Sampling.prototype.getWeight = function() {
        return 0;
    };

    /**
     * @constructor
     * @extends {glsBuiltinPrecisionTests.Sampling}
     * @param {glsBuiltinPrecisionTests.Typename} typename
     */
     glsBuiltinPrecisionTests.DefaultSamplingVoid = function(typename) {
         glsBuiltinPrecisionTests.Sampling.call(this, typename);
     };

     glsBuiltinPrecisionTests.DefaultSamplingVoid.prototype = Object.create(glsBuiltinPrecisionTests.Sampling.prototype);
     glsBuiltinPrecisionTests.DefaultSamplingVoid.prototype.constructor = glsBuiltinPrecisionTests.DefaultSamplingVoid;

    /**
     * @param {tcuFloatFormat.FloatFormat} fmt
     * @param {Array<number>} dst
     */
    glsBuiltinPrecisionTests.DefaultSamplingVoid.prototype.genFixeds = function(fmt, dst) {
        dst.push(NaN);
    };

    /**
     * @constructor
     * @extends {glsBuiltinPrecisionTests.Sampling}
     * @param {glsBuiltinPrecisionTests.Typename} typename
     */
    glsBuiltinPrecisionTests.DefaultSamplingBool = function(typename) {
        glsBuiltinPrecisionTests.Sampling.call(this, typename);
    };

    glsBuiltinPrecisionTests.DefaultSamplingBool.prototype = Object.create(glsBuiltinPrecisionTests.Sampling.prototype);
    glsBuiltinPrecisionTests.DefaultSamplingBool.prototype.constructor = glsBuiltinPrecisionTests.DefaultSamplingBool;

    /**
     * @param {tcuFloatFormat.FloatFormat} fmt
     * @param {Array<Boolean>} dst
     */
    glsBuiltinPrecisionTests.DefaultSamplingBool.prototype.genFixeds = function(fmt, dst) {
        dst.push(true);
        dst.push(false);
    };

    /**
     * @constructor
     * @extends {glsBuiltinPrecisionTests.Sampling}
     * @param {glsBuiltinPrecisionTests.Typename} typename
     */
    glsBuiltinPrecisionTests.DefaultSamplingInt = function(typename) {
        glsBuiltinPrecisionTests.Sampling.call(this, typename);
    };

    glsBuiltinPrecisionTests.DefaultSamplingInt.prototype = Object.create(glsBuiltinPrecisionTests.Sampling.prototype);
    glsBuiltinPrecisionTests.DefaultSamplingInt.prototype.constructor = glsBuiltinPrecisionTests.DefaultSamplingInt;

    glsBuiltinPrecisionTests.DefaultSamplingInt.prototype.genRandom = function(fmt, prec, rnd) {
        /** @type {number} */ var exp = rnd.getInt(0, this.getNumBits(prec) - 2);
        /** @type {number} */ var sign = rnd.getBool() ? -1 : 1;

        return sign * rnd.getInt(0, 1 << exp);
    };

    glsBuiltinPrecisionTests.DefaultSamplingInt.prototype.genFixeds = function(fmt, dst) {
        dst.push(0);
        dst.push(-1);
        dst.push(1);
    };

    glsBuiltinPrecisionTests.DefaultSamplingInt.prototype.getWeight = function() {
        return 1.0;
    };

    /**
     * @param {gluShaderUtil.precision} prec
     * @return {number}
     */
    glsBuiltinPrecisionTests.DefaultSamplingInt.prototype.getNumBits = function(prec) {
        switch (prec) {
            case gluShaderUtil.precision.PRECISION_LOWP: return 8;
            case gluShaderUtil.precision.PRECISION_MEDIUMP: return 16;
            case gluShaderUtil.precision.PRECISION_HIGHP: return 32;
            default:
                throw new Error('Invalid precision: ' + prec);
        }
    };

    /**
     * @constructor
     * @extends {glsBuiltinPrecisionTests.Sampling}
     * @param {glsBuiltinPrecisionTests.Typename} typename
     */
    glsBuiltinPrecisionTests.DefaultSamplingFloat = function(typename) {
        glsBuiltinPrecisionTests.Sampling.call(this, typename);
    };

    glsBuiltinPrecisionTests.DefaultSamplingFloat.prototype = Object.create(glsBuiltinPrecisionTests.Sampling.prototype);
    glsBuiltinPrecisionTests.DefaultSamplingFloat.prototype.constructor = glsBuiltinPrecisionTests.DefaultSamplingFloat;

    glsBuiltinPrecisionTests.DefaultSamplingFloat.prototype.genRandom = function(format, prec, rnd) {
        /** type{number} */ var minExp = format.getMinExp();
        /** type{number} */ var maxExp = format.getMaxExp();
        /** type{boolean} */ var haveSubnormal = format.hasSubnormal() != tcuFloatFormat.YesNoMaybe.NO;

        // Choose exponent so that the cumulative distribution is cubic.
        // This makes the probability distribution quadratic, with the peak centered on zero.
        /** type{number} */ var minRoot = deMath.deCbrt(minExp - 0.5 - (haveSubnormal ? 1.0 : 0.0));
        /** type{number} */ var maxRoot = deMath.deCbrt(maxExp + 0.5);
        /** type{number} */ var fractionBits = format.getFractionBits();
        /** type{number} */ var exp = deMath.rint(Math.pow(rnd.getFloat(minRoot, maxRoot),
                                                                3.0));
        /** type{number} */ var base = 0.0; // integral power of two
        /** type{number} */ var quantum = 0.0; // smallest representable difference in the binade
        /** type{number} */ var significand = 0.0; // Significand.

        // DE_ASSERT(fractionBits < std::numeric_limits<float>::digits);

        // Generate some occasional special numbers
        switch (rnd.getInt(0, 64)) {
            case 0: return 0;
            case 1: return Number.POSITIVE_INFINITY;
            case 2: return Number.NEGATIVE_INFINITY;
            case 3: return NaN;
            default: break;
        }

        if (exp >= minExp) {
            // Normal number
            base = deMath.deFloatLdExp(1.0, exp);
            quantum = deMath.deFloatLdExp(1.0, exp - fractionBits);
        } else {
            // Subnormal
            base = 0.0;
            quantum = deMath.deFloatLdExp(1.0, minExp - fractionBits);
        }

        switch (rnd.getInt(0, 16)) {
            // The highest number in this binade, significand is all bits one.
            case 0:
                significand = base - quantum;
                break;
            // Significand is one.
            case 1:
                significand = quantum;
                break;
            // Significand is zero.
            case 2:
                significand = 0.0;
                break;
            // Random (evenly distributed) significand.
            default: {
                /** type{number} */ var intFraction = rnd.getInt() & ((1 << fractionBits) - 1);
                significand = intFraction * quantum;
            }
        }

        // Produce positive numbers more often than negative.
        return (rnd.getInt(0, 3) == 0 ? -1.0 : 1.0) * (base + significand);
    };

    glsBuiltinPrecisionTests.DefaultSamplingFloat.prototype.genFixeds = function(format, dst) {
        /** @type {number} */ var minExp = format.getMinExp();
        /** @type {number} */ var maxExp = format.getMaxExp();
        /** @type {number} */ var fractionBits = format.getFractionBits();
        /** @type {number} */ var minQuantum = deMath.deFloatLdExp(1.0, minExp - fractionBits);
        /** @type {number} */ var minNormalized = deMath.deFloatLdExp(1.0, minExp);
        /** @type {number} */ var maxQuantum = deMath.deFloatLdExp(1.0, maxExp - fractionBits);

        // If unit testing is enabled, include exact numbers
        if (enableUnittests) {
            dst.push(0.2);
            dst.push(0.5);
        }

        // NaN
        dst.push(NaN);
        // Zero
        dst.push(0.0);

        for (var sign = -1; sign <= 1; sign += 2) {
            // Smallest subnormal
            dst.push(sign * minQuantum);

            // Largest subnormal
            dst.push(sign * (minNormalized - minQuantum));

            // Smallest normalized
            dst.push(sign * minNormalized);

            // Next smallest normalized
            dst.push(sign * (minNormalized + minQuantum));

            dst.push(sign * 0.5);
            dst.push(sign * 1.0);
            dst.push(sign * 2.0);

            // Largest number
            dst.push(sign * (deMath.deFloatLdExp(1.0, maxExp) +
                                  (deMath.deFloatLdExp(1.0, maxExp) - maxQuantum)));

            dst.push(sign * Number.POSITIVE_INFINITY);
        }
    };

    /**
     * @constructor
     * @extends {glsBuiltinPrecisionTests.Sampling}
     * @param {glsBuiltinPrecisionTests.Typename} typename
     * @param {number} size
     */
    glsBuiltinPrecisionTests.DefaultSamplingVector = function(typename, size) {
        glsBuiltinPrecisionTests.Sampling.call(this, typename);
        this.size = size;
    };

    glsBuiltinPrecisionTests.DefaultSamplingVector.prototype = Object.create(glsBuiltinPrecisionTests.Sampling.prototype);
    glsBuiltinPrecisionTests.DefaultSamplingVector.prototype.constructor = glsBuiltinPrecisionTests.DefaultSamplingVector;

    glsBuiltinPrecisionTests.DefaultSamplingVector.prototype.genRandom = function(fmt, prec, rnd) {
        /** @type {Array<*>} */ var ret = [];

        for (var ndx = 0; ndx < this.size; ++ndx)
            ret[ndx] = glsBuiltinPrecisionTests.SamplingFactory(this.typename).genRandom(fmt, prec, rnd);

        return ret;
    };

    glsBuiltinPrecisionTests.DefaultSamplingVector.prototype.genFixeds = function(fmt, dst) {
        /** @type {Array<*>} */ var scalars = [];

        glsBuiltinPrecisionTests.SamplingFactory(this.typename).genFixeds(fmt, scalars);

        for (var scalarNdx = 0; scalarNdx < scalars.length; ++scalarNdx) {
            var value = [];
            for (var i = 0; i < this.size; i++)
                value[i] = scalars[scalarNdx];
            dst.push(value);
        }
    };

    glsBuiltinPrecisionTests.DefaultSamplingVector.prototype.getWeight = function() {
        return Math.pow(glsBuiltinPrecisionTests.SamplingFactory(this.typename).getWeight(), this.size);
    };

    /**
     * @constructor
     * @extends {glsBuiltinPrecisionTests.Sampling}
     * @param {glsBuiltinPrecisionTests.Typename} typename
     * @param {number} rows
     * @param {number} cols
     */
    glsBuiltinPrecisionTests.DefaultSamplingMatrix = function(typename, rows, cols) {
        glsBuiltinPrecisionTests.Sampling.call(this, typename);
        this.rows = rows;
        this.cols = cols;
    };

    glsBuiltinPrecisionTests.DefaultSamplingMatrix.prototype = Object.create(glsBuiltinPrecisionTests.Sampling.prototype);
    glsBuiltinPrecisionTests.DefaultSamplingMatrix.prototype.constructor = glsBuiltinPrecisionTests.DefaultSamplingMatrix;

    glsBuiltinPrecisionTests.DefaultSamplingMatrix.prototype.genRandom = function(fmt, prec, rnd) {
        /** @type {tcuMatrix.Matrix} */ var ret = new tcuMatrix.Matrix(this.rows, this.cols);
        var sampler = glsBuiltinPrecisionTests.SamplingFactory(this.typename);

        for (var rowNdx = 0; rowNdx < this.rows; ++rowNdx)
            for (var colNdx = 0; colNdx < this.cols; ++colNdx)
                ret.set(rowNdx, colNdx, sampler.genRandom(fmt, prec, rnd));

        return ret;
    };

    glsBuiltinPrecisionTests.DefaultSamplingMatrix.prototype.genFixeds = function(fmt, dst) {
        /** @type {Array<number>} */ var scalars = [];

        glsBuiltinPrecisionTests.SamplingFactory(this.typename).genFixeds(fmt, scalars);

        for (var scalarNdx = 0; scalarNdx < scalars.length; ++scalarNdx)
            dst.push(new tcuMatrix.Matrix(this.rows, this.cols, scalars[scalarNdx]));

        if (this.cols == this.rows) {
            var mat = new tcuMatrix.Matrix(this.rows, this.cols, 0);
            var x = 1;
            mat.set(0, 0, x);
            for (var ndx = 0; ndx < this.cols; ++ndx) {
                mat.set(this.cols - 1 - ndx, ndx, x);
                x *= 2;
            }
            dst.push(mat);
        }
    };

    glsBuiltinPrecisionTests.DefaultSamplingMatrix.prototype.getWeight = function() {
        return Math.pow(glsBuiltinPrecisionTests.SamplingFactory(this.typename).getWeight(), this.rows * this.cols);
    };

    /**
     * @constructor
     * @param {number=} size
     * @param {glsBuiltinPrecisionTests.InTypes} In
     */
     glsBuiltinPrecisionTests.Samplings = function(In, size) {
        this.in0 = glsBuiltinPrecisionTests.SamplingFactory(In.In0, size);
        this.in1 = glsBuiltinPrecisionTests.SamplingFactory(In.In1, size);
        this.in2 = glsBuiltinPrecisionTests.SamplingFactory(In.In2, size);
        this.in3 = glsBuiltinPrecisionTests.SamplingFactory(In.In3, size);
    };

    /**
     * @param {glsBuiltinPrecisionTests.InTypes} In
     * @param {number=} size
     * @constructor
     * @extends {glsBuiltinPrecisionTests.Samplings}
     */
     glsBuiltinPrecisionTests.DefaultSamplings = function(In, size) {
        glsBuiltinPrecisionTests.Samplings.call(this, In, size);
    };

    /**
     * @constructor
     * @extends {tcuTestCase.DeqpTest}
     * @param {glsBuiltinPrecisionTests.Context} context
     * @param {string} name
     * @param {string} extension
     */
    glsBuiltinPrecisionTests.PrecisionCase = function(context, name, extension) {
        /** @type {string} */ this.m_extension = extension === undefined ? '' : extension;
        /** @type {glsBuiltinPrecisionTests.Context} */ this.m_ctx = context;
        /** @type {*} */ this.m_status;
        /** @type {deRandom.Random} */ this.m_rnd = new deRandom.Random(1234); //    (0xdeadbeefu + context.testContext.getCommandLine().getBaseSeed())
        tcuTestCase.DeqpTest.call(this, name, extension);
    };

    glsBuiltinPrecisionTests.PrecisionCase.prototype = Object.create(tcuTestCase.DeqpTest.prototype);
    glsBuiltinPrecisionTests.PrecisionCase.prototype.constructor = glsBuiltinPrecisionTests.PrecisionCase;

    /**
     * @return {tcuFloatFormat.FloatFormat}
     */
    glsBuiltinPrecisionTests.PrecisionCase.prototype.getFormat = function() {
        return this.m_ctx.floatFormat;
    };

    /**
     * Return an output value extracted from flat array
     * @param {goog.NumberArray} output
     * @param {number} index Index of the element to extract
     * @param {*} reference Reference for type informaion
     * @return {glsBuiltinPrecisionTests.Value}
     */
    glsBuiltinPrecisionTests.getOutput = function(output, index, reference) {
        if (reference instanceof Array) {
            var ret = [];
            var size = reference.length;
            for (var i = 0; i < size; i++)
                ret[i] = output[size * index + i];
            return ret;
        }
        if (reference instanceof tcuMatrix.Matrix) {
            var ret = new tcuMatrix.Matrix(reference.rows, reference.cols);
            var size = reference.rows * reference.cols;
            for (var i = 0; i < reference.rows; i++)
                for (var j = 0; j < reference.cols; j++)
                    ret.set(i, j, output[size * index + j * reference.rows + i]);
            return ret;
        }

        return output[index];
    };
    /**
     * @param {glsBuiltinPrecisionTests.Variables} variables Variables<In, Out>
     * @param {glsBuiltinPrecisionTests.Inputs} inputs Inputs<In>
     * @param {glsBuiltinPrecisionTests.Statement} stmt
     */
    glsBuiltinPrecisionTests.PrecisionCase.prototype.testStatement = function(variables, inputs, stmt) {
        /** @type {tcuFloatFormat.FloatFormat} */ var fmt = this.getFormat();
        /** @type {number} */ var inCount = glsBuiltinPrecisionTests.numInputs(this.In);
        /** @type {number} */ var outCount = glsBuiltinPrecisionTests.numOutputs(this.Out);
        /** @type {number} */ var numValues = (inCount > 0) ? inputs.in0.length : 1;
        /** @type {tcuFloatFormat.FloatFormat} */ var highpFmt = this.m_ctx.highpFormat;
        var outputs = [];
        /** @type {number} */ var maxMsgs = 100;
        /** @type {number} */ var numErrors = 0;
        /** @type {glsShaderExecUtil.ShaderSpec} */ var spec = new glsShaderExecUtil.ShaderSpec();
        /** @type {glsBuiltinPrecisionTests.Environment} */ var env = new glsBuiltinPrecisionTests.Environment(); // Hoisted out of the inner loop for optimization.

        switch (inCount) {
            case 4: DE_ASSERT(inputs.in3.length == numValues);
            case 3: DE_ASSERT(inputs.in2.length == numValues);
            case 2: DE_ASSERT(inputs.in1.length == numValues);
            case 1: DE_ASSERT(inputs.in0.length == numValues);
            default: break;
        }
        if (enableUnittests)
            numValues = 2;

        // TODO: Fix logging
        //Print out the statement and its definitions
        // bufferedLogToConsole("Statement: " + stmt);
        // var funcInfo = ''
        // var funcs = {};
        // stmt.getUsedFuncs(funcs);
        // for (var key in funcs) {
        //     var func = funcs[key];
        //     funcInfo += func.printDefinition();
        // };
        // if (funcInfo.length > 0)
        //     bufferedLogToConsole('Reference definitions:' + funcInfo);

        // Initialize ShaderSpec from precision, variables and statement.

        spec.globalDeclarations = 'precision ' + gluShaderUtil.getPrecisionName(this.m_ctx.precision) + ' float;\n';

        if (this.m_extension.length > 0)
            spec.globalDeclarations += '#extension ' + this.m_extension + ' : require\n';

        spec.inputs = [];

        switch (inCount) {
            case 4: spec.inputs[3] = this.makeSymbol(variables.in3);
            case 3: spec.inputs[2] = this.makeSymbol(variables.in2);
            case 2: spec.inputs[1] = this.makeSymbol(variables.in1);
            case 1: spec.inputs[0] = this.makeSymbol(variables.in0);
            default: break;
        }

        spec.outputs = [];

        switch (outCount) {
            case 2: spec.outputs[1] = this.makeSymbol(variables.out1);
            case 1: spec.outputs[0] = this.makeSymbol(variables.out0);
            default: break;
        }

        spec.source = stmt;

        if (enableUnittests == false) {
            // Run the shader with inputs.
            /** @type {glsShaderExecUtil.ShaderExecutor} */
            var executor = glsShaderExecUtil.createExecutor(this.m_ctx.shaderType, spec);
            /** @type {Array<*>} */ var inputArr =
            [
            tcuMatrixUtil.flatten(inputs.in0), tcuMatrixUtil.flatten(inputs.in1), tcuMatrixUtil.flatten(inputs.in2), tcuMatrixUtil.flatten(inputs.in3)
            ];

            // executor.log(log());
            if (!executor.isOk())
                testFailed('Shader compilation failed');

            executor.useProgram();
            var outputArray = executor.execute(numValues, inputArr);

            switch (outCount) {
                case 2:
                    outputs[1] = glsBuiltinPrecisionTests.cast(this.Out.Out1, outputArray[1]);
                case 1:
                    outputs[0] = glsBuiltinPrecisionTests.cast(this.Out.Out0, outputArray[0]);
                default: break;
            }
        }

        // Initialize environment with dummy values so we don't need to bind in inner loop.

        var in0 = new tcuInterval.Interval();
        var in1 = new tcuInterval.Interval();
        var in2 = new tcuInterval.Interval();
        var in3 = new tcuInterval.Interval();
        var reference0 = new tcuInterval.Interval();
        var reference1 = new tcuInterval.Interval();

        env.bind(variables.in0, in0);
        env.bind(variables.in1, in1);
        env.bind(variables.in2, in2);
        env.bind(variables.in3, in3);
        env.bind(variables.out0, reference0);
        env.bind(variables.out1, reference1);

        // For each input tuple, compute output reference interval and compare
        // shader output to the reference.
        for (var valueNdx = 0; valueNdx < numValues; valueNdx++) {
            /** @type {boolean} */ var result = true;
            var value0, value1;
            var msg = '';

            var in0_ = glsBuiltinPrecisionTests.convert(this.Arg0, fmt, glsBuiltinPrecisionTests.round(this.Arg0, fmt, inputs.in0[valueNdx]));
            var in1_ = glsBuiltinPrecisionTests.convert(this.Arg1, fmt, glsBuiltinPrecisionTests.round(this.Arg1, fmt, inputs.in1[valueNdx]));
            var in2_ = glsBuiltinPrecisionTests.convert(this.Arg2, fmt, glsBuiltinPrecisionTests.round(this.Arg2, fmt, inputs.in2[valueNdx]));
            var in3_ = glsBuiltinPrecisionTests.convert(this.Arg3, fmt, glsBuiltinPrecisionTests.round(this.Arg3, fmt, inputs.in3[valueNdx]));

            env.bind(variables.in0, in0_);
            env.bind(variables.in1, in1_);
            env.bind(variables.in2, in2_);
            env.bind(variables.in3, in3_);

            stmt.execute(new glsBuiltinPrecisionTests.EvalContext(fmt, this.m_ctx.precision, env));

            switch (outCount) {
                case 2:
                    reference1 = glsBuiltinPrecisionTests.convert(this.Out.Out1, highpFmt, env.lookup(variables.out1));
                    if (enableUnittests)
                        result = referenceComparison(reference1, valueNdx + outCount - 1, this.m_ctx.floatFormat);
                    else {
                        value1 = glsBuiltinPrecisionTests.getOutput(outputs[1], valueNdx, reference1);
                        if (!glsBuiltinPrecisionTests.contains(this.Out.Out1, reference1, value1)) {
                        msg = 'Shader output 1 (' + value1 + ') is outside acceptable range: ' + reference1;
                            result = false;
                        }
                    }
                case 1:
                    reference0 = glsBuiltinPrecisionTests.convert(this.Out.Out0, highpFmt, env.lookup(variables.out0));
                    if (enableUnittests)
                        result = referenceComparison(reference0, valueNdx + outCount - 1, this.m_ctx.floatFormat);
                    else {
                        value0 = glsBuiltinPrecisionTests.getOutput(outputs[0], valueNdx, reference0);
                        if (!glsBuiltinPrecisionTests.contains(this.Out.Out0, reference0, value0)) {
                        msg = 'Shader output 0 (' + value0 + ') is outside acceptable range: ' + reference0;
                            result = false;
                        }
                    }
                default: break;
            }

            if (!result)
                ++numErrors;

            if (!result && numErrors <= maxMsgs) {
                /** @type {string} */ var builder = '';

                builder += (result ? 'Passed' : 'Failed') + '\n' + msg + '\n sample:\n' + valueNdx;

                if (inCount > 0) {
                    builder += '\t' + variables.in0.getName() + ' = ' +
                            inputs.in0[valueNdx] + '\n';
                }

                if (inCount > 1) {
                    builder += '\t' + variables.in1.getName() + ' = ' +
                            inputs.in1[valueNdx] + '\n';
                }

                if (inCount > 2) {
                    builder += '\t' + variables.in2.getName() + ' = ' +
                            inputs.in2[valueNdx] + '\n';
                }

                if (inCount > 3) {
                    builder += '\t' + variables.in3.getName() + ' = ' +
                            inputs.in3[valueNdx] + '\n';
                }

                if (enableUnittests == false) {
                    if (outCount > 0) {
                        builder += '\t' + variables.out0.getName() + ' = ' +
                            value0 + '\n' +
                                '\tExpected range: ' +
                                reference0 + '\n';
                    }

                    if (outCount > 1) {
                        builder += '\t' + variables.out1.getName() + ' = ' +
                            value1 + '\n' +
                                '\tExpected range: ' +
                                reference1 + '\n';
                    }
                }
                bufferedLogToConsole(builder);
            }
        }

        if (numErrors > maxMsgs) {
            bufferedLogToConsole('(Skipped ' + (numErrors - maxMsgs) + ' messages.)');
        }

        if (numErrors == 0) {
            testPassed('All ' + numValues + ' inputs passed.');
        } else {
            testFailed('' + numErrors + '/' + numValues + ' inputs failed.');
        }
    };

    /**
     * @param {glsBuiltinPrecisionTests.Variable} variable Variable<typename>
     * @return {glsShaderExecUtil.Symbol}
     */
    glsBuiltinPrecisionTests.PrecisionCase.prototype.makeSymbol = function(variable) {
        var v = variable;
        return new glsShaderExecUtil.Symbol(v.getName(), gluVarType.getVarTypeOf(v.typename, this.m_size, this.m_ctx.precision));
    };

    /**
     * @constructor
     * @extends {glsBuiltinPrecisionTests.Tuple4}
     * @param {*} in0
     * @param {*} in1
     * @param {*} in2
     * @param {*} in3
     */
    glsBuiltinPrecisionTests.InTuple = function(in0, in1, in2, in3) {
        glsBuiltinPrecisionTests.Tuple4.call(this, in0, in1, in2, in3);
    };

    glsBuiltinPrecisionTests.InTuple.prototype = Object.create(glsBuiltinPrecisionTests.Tuple4.prototype);
    glsBuiltinPrecisionTests.InTuple.prototype.constructor = glsBuiltinPrecisionTests.InTuple;

    /**
     * @param {*} In
     * @param {glsBuiltinPrecisionTests.Samplings} samplings Samplings<In>
     * @param {tcuFloatFormat.FloatFormat} floatFormat
     * @param {gluShaderUtil.precision} intPrecision
     * @param {number} numSamples
     * @param {deRandom.Random} rnd
     * @return {glsBuiltinPrecisionTests.Inputs}
     */
    glsBuiltinPrecisionTests.generateInputs = function(In, samplings, floatFormat, intPrecision, numSamples, rnd) {
        /*Inputs<In>*/ var ret = new glsBuiltinPrecisionTests.Inputs(In);
        /*Inputs<In>*/ var fixedInputs = new glsBuiltinPrecisionTests.Inputs(In);
        // set<InTuple<In>, InputLess<InTuple<In> > > seenInputs;
        /** @type {Array<glsBuiltinPrecisionTests.InTuple,glsBuiltinPrecisionTests.InputLess>} */
        var seenInputs = [];

        samplings.in0.genFixeds(floatFormat, fixedInputs.in0);
        samplings.in1.genFixeds(floatFormat, fixedInputs.in1);
        samplings.in2.genFixeds(floatFormat, fixedInputs.in2);
        samplings.in3.genFixeds(floatFormat, fixedInputs.in3);

        for (var ndx0 = 0; ndx0 < fixedInputs.in0.length; ++ndx0) {
            for (var ndx1 = 0; ndx1 < fixedInputs.in1.length; ++ndx1) {
                for (var ndx2 = 0; ndx2 < fixedInputs.in2.length; ++ndx2) {
                    for (var ndx3 = 0; ndx3 < fixedInputs.in3.length; ++ndx3) {
                        var tuple = new glsBuiltinPrecisionTests.InTuple(fixedInputs.in0[ndx0],
                                                     fixedInputs.in1[ndx1],
                                                     fixedInputs.in2[ndx2],
                                                     fixedInputs.in3[ndx3]);

                        seenInputs.push(tuple);
                        ret.in0.push(tuple.a);
                        ret.in1.push(tuple.b);
                        ret.in2.push(tuple.c);
                        ret.in3.push(tuple.d);
                    }
                }
            }
        }

        for (var ndx = 0; ndx < numSamples; ++ndx) {
            var in0 = samplings.in0.genRandom(floatFormat, intPrecision, rnd);
            var in1 = samplings.in1.genRandom(floatFormat, intPrecision, rnd);
            var in2 = samplings.in2.genRandom(floatFormat, intPrecision, rnd);
            var in3 = samplings.in3.genRandom(floatFormat, intPrecision, rnd);
            var tuple = new glsBuiltinPrecisionTests.InTuple(in0, in1, in2, in3);

            // if (de::contains(seenInputs, tuple))
            //     continue;

            seenInputs.push(tuple);
            ret.in0.push(in0);
            ret.in1.push(in1);
            ret.in2.push(in2);
            ret.in3.push(in3);
        }

        return ret;
    };

    /**
     * @constructor
     * @extends {glsBuiltinPrecisionTests.PrecisionCase}
     * @param {string} name
     * @param {glsBuiltinPrecisionTests.FuncBase} func
     */
    glsBuiltinPrecisionTests.FuncCaseBase = function(context, name, func) {
        glsBuiltinPrecisionTests.PrecisionCase.call(this, context, name, func.getRequiredExtension());
    };

    glsBuiltinPrecisionTests.FuncCaseBase.prototype = Object.create(glsBuiltinPrecisionTests.PrecisionCase.prototype);
    glsBuiltinPrecisionTests.FuncCaseBase.prototype.constructor = glsBuiltinPrecisionTests.FuncCaseBase;

    glsBuiltinPrecisionTests.FuncCaseBase.prototype.iterate = function() {

        assertMsgOptions(!(this.m_extension !== undefined && this.m_extension.trim() !== '') &&
            !sglrGLContext.isExtensionSupported(gl, this.m_extension),
                'Unsupported extension: ' + this.m_extension, false, true);

        this.runTest();

        // m_status.setTestContextResult(m_testCtx);
        return tcuTestCase.IterateResult.STOP;
    };

    /**
     * @constructor
     * @extends {glsBuiltinPrecisionTests.FuncCaseBase}
     * @param {glsBuiltinPrecisionTests.Context} context
     * @param {string} name
     * @param {glsBuiltinPrecisionTests.Func} func
     */
    glsBuiltinPrecisionTests.InOutFuncCase = function(context, name, func) {
        glsBuiltinPrecisionTests.FuncCaseBase.call(this, context, name, func);
        this.Sig = func.Sig;
        this.m_func = func;
        this.Ret = func.Sig.Ret;
        this.Arg0 = func.Sig.Arg0;
        this.Arg1 = func.Sig.Arg1;
        this.Arg2 = func.Sig.Arg2;
        this.Arg3 = func.Sig.Arg3;
        this.In = new glsBuiltinPrecisionTests.InTypes(this.Arg0, this.Arg2, this.Arg3);
        this.Out = new glsBuiltinPrecisionTests.OutTypes(this.Ret, this.Arg1);
        this.m_size = this.m_func.m_size;
    };

    glsBuiltinPrecisionTests.InOutFuncCase.prototype = Object.create(glsBuiltinPrecisionTests.FuncCaseBase.prototype);
    glsBuiltinPrecisionTests.InOutFuncCase.prototype.constructor = glsBuiltinPrecisionTests.InOutFuncCase;

    /**
     * Samplings<In>
     * @return {glsBuiltinPrecisionTests.Samplings}
     */
    glsBuiltinPrecisionTests.InOutFuncCase.prototype.getSamplings = function() {
        return new glsBuiltinPrecisionTests.DefaultSamplings(this.In, this.m_size);
    };

    /**
     * @param {glsBuiltinPrecisionTests.Signature} Sig_
     */
    glsBuiltinPrecisionTests.InOutFuncCase.prototype.runTest = function(Sig_) {
        /** @type {glsBuiltinPrecisionTests.Inputs} */ var inputs = (glsBuiltinPrecisionTests.generateInputs(
                                                    this.In,
                                                    this.getSamplings(),
                                                    this.m_ctx.floatFormat,
                                                    this.m_ctx.precision,
                                                    this.m_ctx.numRandoms,
                                                    this.m_rnd));

        var variables = new glsBuiltinPrecisionTests.Variables(this.In, this.Out);
        // Variables<In, Out> variables;
        //
        variables.out0 = new glsBuiltinPrecisionTests.Variable(this.Out.Out0, 'out0');
        variables.out1 = new glsBuiltinPrecisionTests.Variable(this.Arg1, 'out1');
        variables.in0 = new glsBuiltinPrecisionTests.Variable(this.Arg0, 'in0');
        variables.in1 = new glsBuiltinPrecisionTests.Variable(this.Arg2, 'in1');
        variables.in2 = new glsBuiltinPrecisionTests.Variable(this.Arg3, 'in2');
        variables.in3 = new glsBuiltinPrecisionTests.Variable('void', 'in3');

        var expr = glsBuiltinPrecisionTests.applyVar(this.m_func,
                                       variables.in0, variables.out1,
                                       variables.in1, variables.in2);
        var stmt = glsBuiltinPrecisionTests.variableAssignment(variables.out0, expr);

        this.testStatement(variables, inputs, stmt);
    };

    /**
     * @constructor
     * @extends {glsBuiltinPrecisionTests.FuncCaseBase}
     * @param {glsBuiltinPrecisionTests.Context} context
     * @param {string} name
     * @param {glsBuiltinPrecisionTests.Func} func
     */
    glsBuiltinPrecisionTests.FuncCase = function(context, name, func) {
        glsBuiltinPrecisionTests.FuncCaseBase.call(this, context, name, func);
        this.Sig = func.Sig;
        this.m_func = func;
        this.Ret = func.Sig.Ret;
        this.Arg0 = func.Sig.Arg0;
        this.Arg1 = func.Sig.Arg1;
        this.Arg2 = func.Sig.Arg2;
        this.Arg3 = func.Sig.Arg3;
        this.In = new glsBuiltinPrecisionTests.InTypes(this.Arg0, this.Arg1, this.Arg2, this.Arg3);
        this.Out = new glsBuiltinPrecisionTests.OutTypes(this.Ret);
        this.m_size = this.m_func.m_size;
    };

    glsBuiltinPrecisionTests.FuncCase.prototype = Object.create(glsBuiltinPrecisionTests.FuncCaseBase.prototype);
    glsBuiltinPrecisionTests.FuncCase.prototype.constructor = glsBuiltinPrecisionTests.FuncCase;

    /**
     * Samplings<In>
     * @return {glsBuiltinPrecisionTests.Samplings}
     */
    glsBuiltinPrecisionTests.FuncCase.prototype.getSamplings = function() {
        return new glsBuiltinPrecisionTests.DefaultSamplings(this.In, this.m_size);
    };

    /**
     * @param {glsBuiltinPrecisionTests.Signature} Sig_
     */
    glsBuiltinPrecisionTests.FuncCase.prototype.runTest = function(Sig_) {
        /** @type {glsBuiltinPrecisionTests.Inputs} */ var inputs = (glsBuiltinPrecisionTests.generateInputs(
                                                    this.In,
                                                    this.getSamplings(),
                                                    this.m_ctx.floatFormat,
                                                    this.m_ctx.precision,
                                                    this.m_ctx.numRandoms,
                                                    this.m_rnd));

        var variables = new glsBuiltinPrecisionTests.Variables(this.In, this.Out);
        // Variables<In, Out> variables;
        //
        variables.out0 = new glsBuiltinPrecisionTests.Variable(this.Ret, 'out0');
        variables.out1 = new glsBuiltinPrecisionTests.Variable('void', 'out1');
        variables.in0 = new glsBuiltinPrecisionTests.Variable(this.Arg0, 'in0');
        variables.in1 = new glsBuiltinPrecisionTests.Variable(this.Arg1, 'in1');
        variables.in2 = new glsBuiltinPrecisionTests.Variable(this.Arg2, 'in2');
        variables.in3 = new glsBuiltinPrecisionTests.Variable(this.Arg3, 'in3');

        var expr = glsBuiltinPrecisionTests.applyVar(this.m_func,
                                       variables.in0, variables.in1,
                                       variables.in2, variables.in3);
        var stmt = glsBuiltinPrecisionTests.variableAssignment(variables.out0, expr);

        this.testStatement(variables, inputs, stmt);
    };

    /**
     * @param {glsBuiltinPrecisionTests.Func} func
     * @param {glsBuiltinPrecisionTests.Variable} arg0
     * @param {glsBuiltinPrecisionTests.Variable} arg1
     * @param {glsBuiltinPrecisionTests.Variable} arg2
     * @param {glsBuiltinPrecisionTests.Variable} arg3
     * @return {glsBuiltinPrecisionTests.ApplyVar}
     */
    glsBuiltinPrecisionTests.applyVar = function(func, arg0, arg1, arg2, arg3) {
        return new glsBuiltinPrecisionTests.ApplyVar(func.Sig, func, arg0, arg1, arg2, arg3);
    };

    /**
     * @param {glsBuiltinPrecisionTests.Variable} variable
     * @param {glsBuiltinPrecisionTests.ApplyVar} value
     * @param {boolean} isDeclaration
     */
    glsBuiltinPrecisionTests.variableStatement = function(variable, value, isDeclaration) {
        return new glsBuiltinPrecisionTests.VariableStatement(variable, value, isDeclaration);
    };

    /**
     * @param {glsBuiltinPrecisionTests.Variable} variable
     * @param {glsBuiltinPrecisionTests.ApplyVar} value
     */
    glsBuiltinPrecisionTests.variableAssignment = function(variable, value) {
        return glsBuiltinPrecisionTests.variableStatement(variable, value, false);
    };

    /**
     * @constructor
     */
    glsBuiltinPrecisionTests.CaseFactories = function() {};

    /**
     * @return {Array<glsBuiltinPrecisionTests.CaseFactory>}
     */
    glsBuiltinPrecisionTests.CaseFactories.prototype.getFactories = function() {};

    /**
     * @constructor
     * @extends {glsBuiltinPrecisionTests.CaseFactories}
     */
    glsBuiltinPrecisionTests.BuiltinFuncs = function() {
        /** @type {Array<glsBuiltinPrecisionTests.CaseFactory>} */ this.m_factories = [];
    };

    glsBuiltinPrecisionTests.BuiltinFuncs.prototype = Object.create(glsBuiltinPrecisionTests.CaseFactories.prototype);
    glsBuiltinPrecisionTests.BuiltinFuncs.prototype.constructor = glsBuiltinPrecisionTests.BuiltinFuncs;

    /**
     * @return {Array<glsBuiltinPrecisionTests.CaseFactory>}
     */
    glsBuiltinPrecisionTests.BuiltinFuncs.prototype.getFactories = function() {
        return this.m_factories.slice();
    };

    /**
     * @param {glsBuiltinPrecisionTests.CaseFactory} fact
     */
    glsBuiltinPrecisionTests.BuiltinFuncs.prototype.addFactory = function(fact) {
        this.m_factories.push(fact);
    };

    /**
     * @param {glsBuiltinPrecisionTests.Context} context
     * @param {string} name
     * @param {glsBuiltinPrecisionTests.Func} func
     * @return {glsBuiltinPrecisionTests.PrecisionCase}
     */
    glsBuiltinPrecisionTests.createFuncCase = function(context, name, func) {
        switch (func.getOutParamIndex()) {
            case -1:
                return new glsBuiltinPrecisionTests.FuncCase(context, name, func);
            case 1:
                return new glsBuiltinPrecisionTests.InOutFuncCase(context, name, func);
            default:
                throw new Error(!'Impossible');
        }
    };

    /**
     * @constructor
     */
    glsBuiltinPrecisionTests.CaseFactory = function() {};

    /**
     * @return {string}
     */
    glsBuiltinPrecisionTests.CaseFactory.prototype.getName = function() {
        return '';
    };

    /**
     * @return {string}
     */
    glsBuiltinPrecisionTests.CaseFactory.prototype.getDesc = function() {
        return '';
    };

    /**
     * @param {glsBuiltinPrecisionTests.Context} ctx
     */
    glsBuiltinPrecisionTests.CaseFactory.prototype.createCase = function(ctx) {
        throw new Error('Virtual function. Please override.');
    };

    /**
     * @constructor
     * @extends {glsBuiltinPrecisionTests.CaseFactory}
     * @param {glsBuiltinPrecisionTests.Func} func
     */
    glsBuiltinPrecisionTests.SimpleFuncCaseFactory = function(func) {
        glsBuiltinPrecisionTests.CaseFactory.call(this);
        this.m_func = func;
    };

    setParentClass(glsBuiltinPrecisionTests.SimpleFuncCaseFactory, glsBuiltinPrecisionTests.CaseFactory);

    glsBuiltinPrecisionTests.SimpleFuncCaseFactory.prototype.getName = function() {
        return this.m_func.getName().toLowerCase();
    };

    glsBuiltinPrecisionTests.SimpleFuncCaseFactory.prototype.getDesc = function() {
        return "Function '" + this.getName() + "'";
    };

    glsBuiltinPrecisionTests.SimpleFuncCaseFactory.prototype.createCase = function(ctx) {
        return glsBuiltinPrecisionTests.createFuncCase(ctx, ctx.name, this.m_func);
    };

    /**
     * @constructor
     * @extends {glsBuiltinPrecisionTests.CaseFactory}
     */
    glsBuiltinPrecisionTests.FuncCaseFactory = function() {
        glsBuiltinPrecisionTests.CaseFactory.call(this);
    };

    setParentClass(glsBuiltinPrecisionTests.FuncCaseFactory, glsBuiltinPrecisionTests.CaseFactory);

    glsBuiltinPrecisionTests.FuncCaseFactory.prototype.getFunc = function() {
        throw new Error('Virtual function. Please override.');
    };

    glsBuiltinPrecisionTests.FuncCaseFactory.prototype.getName = function() {
        return this.getFunc().getName().toLowerCase();
    };

    glsBuiltinPrecisionTests.FuncCaseFactory.prototype.getDesc = function() {
        return "Function '" + this.getFunc().getName() + "'";
    };

    /**
     * @constructor
     * @extends {glsBuiltinPrecisionTests.FuncCaseFactory}
     */
    glsBuiltinPrecisionTests.TemplateFuncCaseFactory = function(genF) {
        glsBuiltinPrecisionTests.FuncCaseFactory.call(this);
        this.m_genF = genF;
    };

    setParentClass(glsBuiltinPrecisionTests.TemplateFuncCaseFactory, glsBuiltinPrecisionTests.FuncCaseFactory);

    glsBuiltinPrecisionTests.TemplateFuncCaseFactory.prototype.getFunc = function() {
        return new this.m_genF(1);
    };

    /**
     * @param {glsBuiltinPrecisionTests.Context} ctx
     */
    glsBuiltinPrecisionTests.TemplateFuncCaseFactory.prototype.createCase = function(ctx) {
        var group = tcuTestCase.newTest(ctx.name, ctx.name);
        group.addChild(glsBuiltinPrecisionTests.createFuncCase(ctx, 'scalar', new this.m_genF(1)));
        group.addChild(glsBuiltinPrecisionTests.createFuncCase(ctx, 'vec2', new this.m_genF(2)));
        group.addChild(glsBuiltinPrecisionTests.createFuncCase(ctx, 'vec3', new this.m_genF(3)));
        group.addChild(glsBuiltinPrecisionTests.createFuncCase(ctx, 'vec4', new this.m_genF(4)));

        return group;
    };

    /**
     * @constructor
     * @extends {glsBuiltinPrecisionTests.FuncCaseFactory}
     */
    glsBuiltinPrecisionTests.MatrixFuncCaseFactory = function(genF) {
        glsBuiltinPrecisionTests.FuncCaseFactory.call(this);
        this.m_genF = genF;
    };

    setParentClass(glsBuiltinPrecisionTests.MatrixFuncCaseFactory, glsBuiltinPrecisionTests.FuncCaseFactory);

    glsBuiltinPrecisionTests.MatrixFuncCaseFactory.prototype.getFunc = function() {
        return new this.m_genF(2, 2);
    };

    /**
     * @param {glsBuiltinPrecisionTests.Context} ctx
     */
    glsBuiltinPrecisionTests.MatrixFuncCaseFactory.prototype.createCase = function(ctx) {
        var group = tcuTestCase.newTest(ctx.name, ctx.name);
        this.addCase(ctx, group, 2, 2);
        this.addCase(ctx, group, 3, 2);
        this.addCase(ctx, group, 4, 2);
        this.addCase(ctx, group, 2, 3);
        this.addCase(ctx, group, 3, 3);
        this.addCase(ctx, group, 4, 3);
        this.addCase(ctx, group, 2, 4);
        this.addCase(ctx, group, 3, 4);
        this.addCase(ctx, group, 4, 4);

        return group;
    };

   /**
     * @param {glsBuiltinPrecisionTests.Context} ctx
     * @param {tcuTestCase.DeqpTest} group
     * @param {number} rows
     * @param {number} cols
     */
    glsBuiltinPrecisionTests.MatrixFuncCaseFactory.prototype.addCase = function(ctx, group, rows, cols) {
        var name = glsBuiltinPrecisionTests.dataTypeNameOfMatrix('float', rows, cols);
        group.addChild(glsBuiltinPrecisionTests.createFuncCase(ctx, name, new this.m_genF(rows, cols)));
    };

    glsBuiltinPrecisionTests.dataTypeNameOfMatrix = function(typename, rows, cols) {
        switch (typename) {
            case 'float':
                if (rows === cols)
                    return 'mat' + rows;
                else
                    return 'mat' + cols + 'x' + rows;
        }
        throw new Error('Invalid arguments (' + typename + ', ' + rows + ', ' + cols + ')');
    };

    /**
     * @constructor
     * @extends {glsBuiltinPrecisionTests.FuncCaseFactory}
     */
    glsBuiltinPrecisionTests.SquareMatrixFuncCaseFactory = function(genF) {
        glsBuiltinPrecisionTests.FuncCaseFactory.call(this);
        this.m_genF = genF;
    };

    setParentClass(glsBuiltinPrecisionTests.SquareMatrixFuncCaseFactory, glsBuiltinPrecisionTests.FuncCaseFactory);

    glsBuiltinPrecisionTests.SquareMatrixFuncCaseFactory.prototype.getFunc = function() {
        return new this.m_genF(2);
    };

    /**
     * @param {glsBuiltinPrecisionTests.Context} ctx
     */
    glsBuiltinPrecisionTests.SquareMatrixFuncCaseFactory.prototype.createCase = function(ctx) {
        var group = tcuTestCase.newTest(ctx.name, ctx.name);

        group.addChild(glsBuiltinPrecisionTests.createFuncCase(ctx, 'mat2', new this.m_genF(2)));
        return group;
    };

    /**
     * @constructor
     * @extends {glsBuiltinPrecisionTests.PrimitiveFunc}
     * @param {glsBuiltinPrecisionTests.Func} scalarFunc
     * @param {number=} size
     */
    glsBuiltinPrecisionTests.GenFunc = function(scalarFunc, size) {
        glsBuiltinPrecisionTests.PrimitiveFunc.call(this, scalarFunc.Sig);
        this.m_func = scalarFunc;
        this.m_size = size;
    };

    glsBuiltinPrecisionTests.GenFunc.prototype = Object.create(glsBuiltinPrecisionTests.PrimitiveFunc.prototype);
    glsBuiltinPrecisionTests.GenFunc.prototype.constructor = glsBuiltinPrecisionTests.GenFunc;

    /**
     * @return {string}
     */
    glsBuiltinPrecisionTests.GenFunc.prototype.getName = function() {
       return this.m_func.getName();
    };

    /**
     * @return {number}
     */
    glsBuiltinPrecisionTests.GenFunc.prototype.getOutParamIndex = function() {
       return this.m_func.getOutParamIndex();
    };

    /**
     * @return {string}
     */
    glsBuiltinPrecisionTests.GenFunc.prototype.getRequiredExtension = function() {
       return this.m_func.getRequiredExtension();
    };

    /**
     * @param {Array<glsBuiltinPrecisionTests.ExprBase>} args
     */
    glsBuiltinPrecisionTests.GenFunc.prototype.doPrint = function(args) {
       return this.m_func.print(args);
    };

    /**
     * @param {glsBuiltinPrecisionTests.EvalContext} ctx
     * @param {glsBuiltinPrecisionTests.Tuple4} iargs
     * @return {*}
     */
    glsBuiltinPrecisionTests.GenFunc.prototype.doApply = function(ctx, iargs) {
        /** @type {Array<*>} */ var ret = [];

        if (this.m_size > 1) {
            for (var ndx = 0; ndx < this.m_size; ++ndx) {
                var a = iargs.a === undefined ? undefined : iargs.a[ndx];
                var b = iargs.b === undefined ? undefined : iargs.b[ndx];
                var c = iargs.c === undefined ? undefined : iargs.c[ndx];
                var d = iargs.d === undefined ? undefined : iargs.d[ndx];
                ret[ndx] = this.m_func.applyFunction(ctx, a, b, c, d);
            }
        } else
            ret[0] = this.m_func.applyFunction(ctx, iargs.a, iargs.b, iargs.c, iargs.d);

        return ret;
    };

    /**
     * @param {glsBuiltinPrecisionTests.FuncSet} dst
     */
    glsBuiltinPrecisionTests.GenFunc.prototype.doGetUsedFuncs = function(dst) {
        this.m_func.getUsedFuncs(dst);
    };

    /**
     * @constructor
     * @extends {glsBuiltinPrecisionTests.GenFunc}
     * @param {glsBuiltinPrecisionTests.Func} func
     * @param {number} size
     */
     glsBuiltinPrecisionTests.VectorizedFunc = function(func, size) {
         glsBuiltinPrecisionTests.GenFunc.call(this, func, size);
    };

    glsBuiltinPrecisionTests.VectorizedFunc.prototype = Object.create(glsBuiltinPrecisionTests.GenFunc.prototype);
    glsBuiltinPrecisionTests.VectorizedFunc.prototype.constructor = glsBuiltinPrecisionTests.VectorizedFunc;

    /**
     * @constructor
     * @param {glsBuiltinPrecisionTests.Func} func_
     * @param {glsBuiltinPrecisionTests.GenFunc} func2_
     * @param {glsBuiltinPrecisionTests.GenFunc} func3_
     * @param {glsBuiltinPrecisionTests.GenFunc} func4_
     */
    glsBuiltinPrecisionTests.GenFuncs = function(func_, func2_, func3_, func4_) {
        this.func = func_;
        this.func2 = func2_;
        this.func3 = func3_;
        this.func4 = func4_;
    };

    /**
     * @constructor
     * @extends {glsBuiltinPrecisionTests.CaseFactory}
     * @param {glsBuiltinPrecisionTests.GenFuncs} funcs
     * @param {string} name
     */
    glsBuiltinPrecisionTests.GenFuncCaseFactory = function(funcs, name) {
        glsBuiltinPrecisionTests.CaseFactory.call(this);
        this.m_funcs = funcs;
        this.m_name = name;
    };

    glsBuiltinPrecisionTests.GenFuncCaseFactory.prototype = Object.create(glsBuiltinPrecisionTests.CaseFactory.prototype);
    glsBuiltinPrecisionTests.GenFuncCaseFactory.prototype.constructor = glsBuiltinPrecisionTests.GenFuncCaseFactory;

    /**
     * @param {glsBuiltinPrecisionTests.Context} ctx
     * @return {tcuTestCase.DeqpTest}
     */
    glsBuiltinPrecisionTests.GenFuncCaseFactory.prototype.createCase = function(ctx) {
        /** @type {tcuTestCase.DeqpTest} */
        var group = tcuTestCase.newTest(ctx.name, ctx.name);
        group.addChild(glsBuiltinPrecisionTests.createFuncCase(ctx, 'scalar', this.m_funcs.func));
        group.addChild(glsBuiltinPrecisionTests.createFuncCase(ctx, 'vec2', this.m_funcs.func2));
        group.addChild(glsBuiltinPrecisionTests.createFuncCase(ctx, 'vec3', this.m_funcs.func3));
        group.addChild(glsBuiltinPrecisionTests.createFuncCase(ctx, 'vec4', this.m_funcs.func4));

        return group;
    };

    /**
     * @return {string}
     */
    glsBuiltinPrecisionTests.GenFuncCaseFactory.prototype.getName = function() {
        return this.m_name;
    };

    /**
     * @return {string}
     */
    glsBuiltinPrecisionTests.GenFuncCaseFactory.prototype.getDesc = function() {
        return "Function '" + this.m_funcs.func.getName() + "'";
    };

    /**
     * @constructor
     * @param {string} name_
     * @param {tcuFloatFormat.FloatFormat} floatFormat_
     * @param {tcuFloatFormat.FloatFormat} highpFormat_
     * @param {gluShaderUtil.precision} precision_
     * @param {gluShaderProgram.shaderType} shaderType_
     * @param {number} numRandoms_
     */
    glsBuiltinPrecisionTests.Context = function(name_, floatFormat_, highpFormat_, precision_, shaderType_, numRandoms_) {
        /** @type {string} */ this.name = name_;
        /** @type {tcuFloatFormat.FloatFormat} */ this.floatFormat = floatFormat_;
        /** @type {tcuFloatFormat.FloatFormat} */ this.highpFormat = highpFormat_;
        /** @type {gluShaderUtil.precision} */ this.precision = precision_;
        /** @type {gluShaderProgram.shaderType} */ this.shaderType = shaderType_;
        /** @type {number} */ this.numRandoms = numRandoms_;
    };

    /**
     * @constructor
     * @param {tcuFloatFormat.FloatFormat} highp_
     * @param {tcuFloatFormat.FloatFormat} mediump_
     * @param {tcuFloatFormat.FloatFormat} lowp_
     * @param {Array<gluShaderProgram.shaderType>} shaderTypes_
     * @param {number} numRandoms_
     */
    glsBuiltinPrecisionTests.PrecisionTestContext = function(highp_, mediump_, lowp_, shaderTypes_, numRandoms_) {
        /** @type {Array<gluShaderProgram.shaderType>} */ this.shaderTypes = shaderTypes_;
        /** @type {Array<tcuFloatFormat.FloatFormat>} */ this.formats = [];
        this.formats[gluShaderUtil.precision.PRECISION_HIGHP] = highp_;
        this.formats[gluShaderUtil.precision.PRECISION_MEDIUMP] = mediump_;
        this.formats[gluShaderUtil.precision.PRECISION_LOWP] = lowp_;
        /** @type {number} */ this.numRandoms = numRandoms_;
    };

    /**
     * \brief Simple incremental counter.
     *
     * This is used to make sure that different ExpandContexts will not produce
     * overlapping temporary names.
     * @constructor
     *
     */
    glsBuiltinPrecisionTests.Counter = function() {
        this.m_count = 0;
    };

    glsBuiltinPrecisionTests.Counter.prototype.get = function() {
        return this.m_count++;
    };

    /**
     * @constructor
     */
    glsBuiltinPrecisionTests.ExpandContext = function(counter) {
        this.m_counter = counter;
        this.m_statements = [];
    };

    /**
     * @param {string} typename
     * @param {string} baseName
     * @return {glsBuiltinPrecisionTests.Variable}
     */
    glsBuiltinPrecisionTests.ExpandContext.prototype.genSym = function(typename, baseName) {
        return new glsBuiltinPrecisionTests.Variable(typename, baseName + this.m_counter.get());
    };

    glsBuiltinPrecisionTests.ExpandContext.prototype.addStatement = function(/*const StatementP&*/ stmt) {
        this.m_statements.push(stmt);
    };

    glsBuiltinPrecisionTests.ExpandContext.prototype.getStatements = function() {
        return this.m_statements;
    };

    /**
     * @constructor
     * @extends {glsBuiltinPrecisionTests.Func}
     * @param {glsBuiltinPrecisionTests.Signature} Sig_ template <typename Sig_>
     */
    glsBuiltinPrecisionTests.DerivedFunc = function(Sig_) {
        glsBuiltinPrecisionTests.Func.call(this, Sig_);
    };

    setParentClass(glsBuiltinPrecisionTests.DerivedFunc, glsBuiltinPrecisionTests.Func);

    glsBuiltinPrecisionTests.DerivedFunc.prototype.doPrintDefinition = function() {
        var os = '';
        var paramNames = this.getParamNames();

        this.initialize();

        os += this.Ret + ' ' + this.getName() +
            '(';
        if (glsBuiltinPrecisionTests.isTypeValid(this.Arg0))
            os += this.Arg0 + ' ' + paramNames.a;
        if (glsBuiltinPrecisionTests.isTypeValid(this.Arg1))
            os += ', ' + this.Arg1 + ' ' + paramNames.b;
        if (glsBuiltinPrecisionTests.isTypeValid(this.Arg2))
            os += ', ' + this.Arg2 + ' ' + paramNames.c;
        if (glsBuiltinPrecisionTests.isTypeValid(this.Arg3))
            os += ', ' + this.Arg3 + ' ' + paramNames.d;
        os += ')\n{\n';

        for (var ndx = 0; ndx < this.m_body.length; ++ndx)
            os += this.m_body[ndx];
        os += 'return ' + this.m_ret + ';\n';
        os += '}\n';

        return os;
    };

    glsBuiltinPrecisionTests.DerivedFunc.prototype.doApply = function(ctx, args) {
        var funEnv = new glsBuiltinPrecisionTests.Environment();
        this.initialize();

        funEnv.bind(this.m_var0, args.a);
        funEnv.bind(this.m_var1, args.b);
        funEnv.bind(this.m_var2, args.c);
        funEnv.bind(this.m_var3, args.d);

        var funCtx = new glsBuiltinPrecisionTests.EvalContext(ctx.format, ctx.floatPrecision, funEnv, ctx.callDepth);

        for (var ndx = 0; ndx < this.m_body.length; ++ndx)
            this.m_body[ndx].execute(funCtx);

        var ret = this.m_ret.evaluate(funCtx);

        // \todo [lauri] Store references instead of values in environment
        args.a = funEnv.lookup(this.m_var0);
        args.b = funEnv.lookup(this.m_var1);
        args.c = funEnv.lookup(this.m_var2);
        args.d = funEnv.lookup(this.m_var3);

        return ret;
    };

    glsBuiltinPrecisionTests.DerivedFunc.prototype.initialize = function() {
        if (!this.m_ret) {
            var paramNames = this.getParamNames();
            var symCounter = new glsBuiltinPrecisionTests.Counter();
            var ctx = new glsBuiltinPrecisionTests.ExpandContext(symCounter);

            this.m_var0 = new glsBuiltinPrecisionTests.Variable(this.Arg0, paramNames.a);
            this.m_var1 = new glsBuiltinPrecisionTests.Variable(this.Arg1, paramNames.b);
            this.m_var2 = new glsBuiltinPrecisionTests.Variable(this.Arg2, paramNames.c);
            this.m_var3 = new glsBuiltinPrecisionTests.Variable(this.Arg3, paramNames.d);
            var args = new glsBuiltinPrecisionTests.Tuple4(this.m_var0,
                this.m_var1, this.m_var2, this.m_var3);

            this.m_ret = this.doExpand(ctx, args);
            this.m_body = ctx.getStatements();
        }
    };

    /**
     * @constructor
     * @extends {glsBuiltinPrecisionTests.Func}
     * @param {glsBuiltinPrecisionTests.Signature} Sig_ template <typename Sig_>
     */
    glsBuiltinPrecisionTests.Alternatives = function(Sig_) {
         glsBuiltinPrecisionTests.Func.call(this, Sig_);
    };

    setParentClass(glsBuiltinPrecisionTests.Alternatives,glsBuiltinPrecisionTests.Func);

    glsBuiltinPrecisionTests.Alternatives.prototype.getName = function() {
        return 'alternatives';
    };

    glsBuiltinPrecisionTests.Alternatives.prototype.doPrintDefinition = function() {};

    glsBuiltinPrecisionTests.Alternatives.prototype.doGetUsedFuncs = function(dst) {};

    glsBuiltinPrecisionTests.Alternatives.prototype.doApply = function(ctx,args) {
        return glsBuiltinPrecisionTests.union(this.Sig.Ret,args.a,args.b);
    };

    glsBuiltinPrecisionTests.Alternatives.prototype.doPrint = function(args) {
        return '{' + args[0] + '|' + args[1] + '}';
    };

    glsBuiltinPrecisionTests.sizeToName = function(size) {
        switch (size) {
            case 4: return 'vec4';
            case 3: return 'vec3';
            case 2: return 'vec2';
        }
        return 'float';
    };

    /**
     * @constructor
     * @param {number} size
     * @extends {glsBuiltinPrecisionTests.DerivedFunc}
     */
    glsBuiltinPrecisionTests.Dot = function(size) {
        var name = glsBuiltinPrecisionTests.sizeToName(size);
        var sig = new glsBuiltinPrecisionTests.Signature('float', name, name);
        glsBuiltinPrecisionTests.DerivedFunc.call(this, sig);
        this.m_inputSize = size;
    };

    setParentClass(glsBuiltinPrecisionTests.Dot, glsBuiltinPrecisionTests.DerivedFunc);

    glsBuiltinPrecisionTests.Dot.prototype.getName = function() {
        return 'dot';
    };

    glsBuiltinPrecisionTests.Dot.prototype.doExpand = function(ctx, args) {
        if (this.m_inputSize > 1) {
            var val = app(new glsBuiltinPrecisionTests.Mul(),
                new glsBuiltinPrecisionTests.VectorVariable(args.a, 0), new glsBuiltinPrecisionTests.VectorVariable(args.b, 0));
            for (var i = 1; i < this.m_inputSize; i++) {
                var tmp = new glsBuiltinPrecisionTests.Apply('float', new glsBuiltinPrecisionTests.Mul(),
                    new glsBuiltinPrecisionTests.VectorVariable(args.a, i), new glsBuiltinPrecisionTests.VectorVariable(args.b, i));
                val = app(new glsBuiltinPrecisionTests.Add(), val, tmp);
            }
            return val;
        } else {
            // args.a * args.b
            var ret = app(new glsBuiltinPrecisionTests.Mul(), args.a, args.b);
            return ret;
        }
    };

    /**
     * @constructor
     * @param {number} size
     * @extends {glsBuiltinPrecisionTests.DerivedFunc}
     */
    glsBuiltinPrecisionTests.Length = function(size) {
        var name = glsBuiltinPrecisionTests.sizeToName(size);
        var sig = new glsBuiltinPrecisionTests.Signature('float', name);
        glsBuiltinPrecisionTests.DerivedFunc.call(this, sig);
        this.m_inputSize = size;
    };

    setParentClass(glsBuiltinPrecisionTests.Length, glsBuiltinPrecisionTests.DerivedFunc);

    glsBuiltinPrecisionTests.Length.prototype.getName = function() {
        return 'length';
    };

    glsBuiltinPrecisionTests.Length.prototype.doExpand = function(ctx, args) {
        //sqrt(dot(args.a, args.a));
        var v0 = app(new glsBuiltinPrecisionTests.Dot(this.m_inputSize), args.a, args.a);
        var v1 = app(new glsBuiltinPrecisionTests.Sqrt(), v0);
        return v1;
    };

    /**
     * @constructor
     * @param {number} size
     * @extends {glsBuiltinPrecisionTests.DerivedFunc}
     */
    glsBuiltinPrecisionTests.Distance = function(size) {
        var name = glsBuiltinPrecisionTests.sizeToName(size);
        var sig = new glsBuiltinPrecisionTests.Signature('float', name, name);
        glsBuiltinPrecisionTests.DerivedFunc.call(this, sig);
        this.m_inputSize = size;
    };

    setParentClass(glsBuiltinPrecisionTests.Distance, glsBuiltinPrecisionTests.DerivedFunc);

    glsBuiltinPrecisionTests.Distance.prototype.getName = function() {
        return 'distance';
    };

    glsBuiltinPrecisionTests.Distance.prototype.doExpand = function(ctx, args) {
        //length(args.a - args.b);
        var v0 = new glsBuiltinPrecisionTests.ApplyScalar(new glsBuiltinPrecisionTests.Sub(), args.a, args.b);
        var v1 = app(new glsBuiltinPrecisionTests.Length(this.m_inputSize), v0);
        return v1;
    };

   /**
     * @constructor
     * @extends {glsBuiltinPrecisionTests.DerivedFunc}
     */
    glsBuiltinPrecisionTests.Cross = function() {
        var sig = new glsBuiltinPrecisionTests.Signature('vec3', 'vec3', 'vec3');
        glsBuiltinPrecisionTests.DerivedFunc.call(this, sig);
        this.m_inputSize = 3;
    };

    setParentClass(glsBuiltinPrecisionTests.Cross, glsBuiltinPrecisionTests.DerivedFunc);

    glsBuiltinPrecisionTests.Cross.prototype.getName = function() {
        return 'cross';
    };

    glsBuiltinPrecisionTests.Cross.prototype.doExpand = function(ctx, args) {
        // vec3(x.a[1] * x.b[2] - x.b[1] * x.a[2],
        //      x.a[2] * x.b[0] - x.b[2] * x.a[0],
        //      x.a[0] * x.b[1] - x.b[0] * x.a[1]);
        var a = [], b = [];
        for (var i = 0; i < this.m_inputSize; i++) {
            a[i] = new glsBuiltinPrecisionTests.VectorVariable(args.a, i);
            b[i] = new glsBuiltinPrecisionTests.VectorVariable(args.b, i);
        }
        var v0 = app(new glsBuiltinPrecisionTests.Mul(), a[1], b[2]);
        var v1 = app(new glsBuiltinPrecisionTests.Mul(), b[1], a[2]);
        var v2 = app(new glsBuiltinPrecisionTests.Sub(), v0, v1);

        var v3 = app(new glsBuiltinPrecisionTests.Mul(), a[2], b[0]);
        var v4 = app(new glsBuiltinPrecisionTests.Mul(), b[2], a[0]);
        var v5 = app(new glsBuiltinPrecisionTests.Sub(), v3, v4);

        var v6 = app(new glsBuiltinPrecisionTests.Mul(), a[0], b[1]);
        var v7 = app(new glsBuiltinPrecisionTests.Mul(), b[0], a[1]);
        var v8 = app(new glsBuiltinPrecisionTests.Sub(), v6, v7);

        var v9 = app(new glsBuiltinPrecisionTests.GenVec(this.m_inputSize, true), v2, v5, v8);
        return v9;
    };

    /**
     * @constructor
     * @param {number} size
     * @extends {glsBuiltinPrecisionTests.DerivedFunc}
     */
    glsBuiltinPrecisionTests.Normalize = function(size) {
        var name = glsBuiltinPrecisionTests.sizeToName(size);
        var sig = new glsBuiltinPrecisionTests.Signature(name, name);
        glsBuiltinPrecisionTests.DerivedFunc.call(this, sig);
        this.m_inputSize = size;
    };

    setParentClass(glsBuiltinPrecisionTests.Normalize, glsBuiltinPrecisionTests.DerivedFunc);

    glsBuiltinPrecisionTests.Normalize.prototype.getName = function() {
        return 'normalize';
    };

    glsBuiltinPrecisionTests.Normalize.prototype.doExpand = function(ctx, args) {
        //args.a / length<Size>(args.a);
        var v0 = app(new glsBuiltinPrecisionTests.Length(this.m_inputSize), args.a);
        var v1 = new glsBuiltinPrecisionTests.ApplyScalar(new glsBuiltinPrecisionTests.Div(), args.a, v0);
        return v1;
    };

    /**
     * @constructor
     * @param {number} size
     * @extends {glsBuiltinPrecisionTests.DerivedFunc}
     */
    glsBuiltinPrecisionTests.FaceForward = function(size) {
        var name = glsBuiltinPrecisionTests.sizeToName(size);
        var sig = new glsBuiltinPrecisionTests.Signature(name, name, name, name);
        glsBuiltinPrecisionTests.DerivedFunc.call(this, sig);
        this.m_inputSize = size;
        this.typename = name;
    };

    setParentClass(glsBuiltinPrecisionTests.FaceForward, glsBuiltinPrecisionTests.DerivedFunc);

    glsBuiltinPrecisionTests.FaceForward.prototype.getName = function() {
        return 'faceforward';
    };

    glsBuiltinPrecisionTests.FaceForward.prototype.doExpand = function(ctx, args) {
        //cond(dot(args.c, args.b) < constant(0.0f), args.a, -args.a);
        var zero = new glsBuiltinPrecisionTests.Constant(0);
        var v0 = new glsBuiltinPrecisionTests.ApplyScalar(new glsBuiltinPrecisionTests.Negate(), args.a);
        var v1 = app(new glsBuiltinPrecisionTests.Dot(this.m_inputSize), args.c, args.b);
        var v2 = app(new glsBuiltinPrecisionTests.LessThan('float'), v1, zero);
        var v3 = app(new glsBuiltinPrecisionTests.Cond(this.typename), v2, args.a, v0);
        return v3;
    };

    /**
     * @constructor
     * @param {number} size
     * @extends {glsBuiltinPrecisionTests.DerivedFunc}
     */
    glsBuiltinPrecisionTests.Reflect = function(size) {
        var name = glsBuiltinPrecisionTests.sizeToName(size);
        var sig = new glsBuiltinPrecisionTests.Signature(name, name, name);
        glsBuiltinPrecisionTests.DerivedFunc.call(this, sig);
        this.m_inputSize = size;
    };

    setParentClass(glsBuiltinPrecisionTests.Reflect, glsBuiltinPrecisionTests.DerivedFunc);

    glsBuiltinPrecisionTests.Reflect.prototype.getName = function() {
        return 'reflect';
    };

    glsBuiltinPrecisionTests.Reflect.prototype.doExpand = function(ctx, args) {
        //args.a - (args.b * dot(args.b, args.a) * constant(2.0f));
        var two = new glsBuiltinPrecisionTests.Constant(2);
        var v0 = app(new glsBuiltinPrecisionTests.Dot(this.m_inputSize), args.b, args.a);
        var v1 = new glsBuiltinPrecisionTests.ApplyScalar(new glsBuiltinPrecisionTests.Mul(), args.b, v0);
        var v2 = new glsBuiltinPrecisionTests.ApplyScalar(new glsBuiltinPrecisionTests.Mul(), v1, two);
        var v3 = new glsBuiltinPrecisionTests.ApplyScalar(new glsBuiltinPrecisionTests.Sub(), args.a, v2);
        return v3;
    };

    /**
     * @constructor
     * @param {number} size
     * @extends {glsBuiltinPrecisionTests.DerivedFunc}
     */
    glsBuiltinPrecisionTests.Refract = function(size) {
        var name = glsBuiltinPrecisionTests.sizeToName(size);
        var sig = new glsBuiltinPrecisionTests.Signature(name, name, name, 'float');
        glsBuiltinPrecisionTests.DerivedFunc.call(this, sig);
        this.m_inputSize = size;
        this.typename = name;
    };

    setParentClass(glsBuiltinPrecisionTests.Refract, glsBuiltinPrecisionTests.DerivedFunc);

    glsBuiltinPrecisionTests.Refract.prototype.getName = function() {
        return 'refract';
    };

    glsBuiltinPrecisionTests.Refract.prototype.doExpand = function(ctx, args) {
        var i = args.a;
        var n = args.b;
        var eta = args.c;
        var zero = new glsBuiltinPrecisionTests.Constant(0);
        var one = new glsBuiltinPrecisionTests.Constant(1);
        // dotNI = dot(n, i)
        var v0 = app(new glsBuiltinPrecisionTests.Dot(this.m_inputSize), n, i);
        var dotNI = glsBuiltinPrecisionTests.bindExpression('float', 'dotNI', ctx, v0);
        // k = 1 - eta * eta * (1 - dotNI * dotNI)
        var v1 = new glsBuiltinPrecisionTests.ApplyScalar(new glsBuiltinPrecisionTests.Mul(), dotNI, dotNI);
        var v2 = new glsBuiltinPrecisionTests.ApplyScalar(new glsBuiltinPrecisionTests.Sub(), one, v1);
        var v3 = new glsBuiltinPrecisionTests.ApplyScalar(new glsBuiltinPrecisionTests.Mul(), eta, eta);
        var v4 = new glsBuiltinPrecisionTests.ApplyScalar(new glsBuiltinPrecisionTests.Mul(), v3, v2);
        var v5 = new glsBuiltinPrecisionTests.ApplyScalar(new glsBuiltinPrecisionTests.Sub(), one, v4);
        var k = glsBuiltinPrecisionTests.bindExpression('float', 'k', ctx, v5);

        // i * eta - n * (eta * dotNI + sqrt(k))
        var v6 = new glsBuiltinPrecisionTests.ApplyScalar(new glsBuiltinPrecisionTests.Mul(), eta, dotNI);
        var v7 = new glsBuiltinPrecisionTests.ApplyScalar(new glsBuiltinPrecisionTests.Sqrt(), k);
        var v8 = new glsBuiltinPrecisionTests.ApplyScalar(new glsBuiltinPrecisionTests.Add(), v6, v7);
        var v9 = new glsBuiltinPrecisionTests.ApplyScalar(new glsBuiltinPrecisionTests.Mul(), n, v8);
        var v10 = new glsBuiltinPrecisionTests.ApplyScalar(new glsBuiltinPrecisionTests.Mul(), i, eta);
    var v11 = new glsBuiltinPrecisionTests.ApplyScalar(new glsBuiltinPrecisionTests.Sub(), v10, v9);

        var v12 = new glsBuiltinPrecisionTests.ApplyScalar(new glsBuiltinPrecisionTests.LessThan('float'), k, zero);

        var zeroVector = app(new glsBuiltinPrecisionTests.GenVec(this.m_inputSize), zero);
        var v13 = app(new glsBuiltinPrecisionTests.Cond(this.typename), v12, zeroVector, v11);
        return v13;
    };

    /**
     * @constructor
     * @extends {glsBuiltinPrecisionTests.DerivedFunc}
     */
    glsBuiltinPrecisionTests.Radians = function() {
        var sig = new glsBuiltinPrecisionTests.Signature('float', 'float');
        glsBuiltinPrecisionTests.DerivedFunc.call(this, sig);
    };

    setParentClass(glsBuiltinPrecisionTests.Radians, glsBuiltinPrecisionTests.DerivedFunc);

    glsBuiltinPrecisionTests.Radians.prototype.getName = function() {
        return 'radians';
    };

    glsBuiltinPrecisionTests.Radians.prototype.doExpand = function(ctx, args) {
        var val = app(new glsBuiltinPrecisionTests.Div(),
                                                      new glsBuiltinPrecisionTests.Constant(Math.PI),
                                                      new glsBuiltinPrecisionTests.Constant(180));
        return new glsBuiltinPrecisionTests.Apply('float',
                                                  new glsBuiltinPrecisionTests.Mul(),
                                                  val,
                                                  args.a);

    };

    /**
     * @constructor
     * @extends {glsBuiltinPrecisionTests.DerivedFunc}
     */
    glsBuiltinPrecisionTests.Degrees = function() {
        var sig = new glsBuiltinPrecisionTests.Signature('float', 'float');
        glsBuiltinPrecisionTests.DerivedFunc.call(this, sig);
    };

    setParentClass(glsBuiltinPrecisionTests.Degrees, glsBuiltinPrecisionTests.DerivedFunc);

    glsBuiltinPrecisionTests.Degrees.prototype.getName = function() {
        return 'degrees';
    };

    glsBuiltinPrecisionTests.Degrees.prototype.doExpand = function(ctx, args) {
        var val = app(new glsBuiltinPrecisionTests.Div(),
                                                      new glsBuiltinPrecisionTests.Constant(180),
                                                      new glsBuiltinPrecisionTests.Constant(Math.PI));
        return new glsBuiltinPrecisionTests.Apply('float',
                                                  new glsBuiltinPrecisionTests.Mul(),
                                                  val,
                                                  args.a);

    };

    /**
     * @constructor
     * @extends {glsBuiltinPrecisionTests.DerivedFunc}
     */
    glsBuiltinPrecisionTests.Sinh = function() {
        var sig = new glsBuiltinPrecisionTests.Signature('float', 'float');
        glsBuiltinPrecisionTests.DerivedFunc.call(this, sig);
    };

    setParentClass(glsBuiltinPrecisionTests.Sinh, glsBuiltinPrecisionTests.DerivedFunc);

    glsBuiltinPrecisionTests.Sinh.prototype.getName = function() {
        return 'sinh';
    };

    glsBuiltinPrecisionTests.Sinh.prototype.doExpand = function(ctx, args) {
        // (exp(x) - exp(-x)) / constant(2.0f)
        var x = args.a;
        var v0 = app(new glsBuiltinPrecisionTests.Exp(), x);
        var v1 = app(new glsBuiltinPrecisionTests.Negate(), x);
        var v2 = app(new glsBuiltinPrecisionTests.Exp(), v1);
        var v3 = app(new glsBuiltinPrecisionTests.Sub(), v0, v2);
        var v4 = new glsBuiltinPrecisionTests.Constant(2);
        var v5 = new glsBuiltinPrecisionTests.Apply('float', new glsBuiltinPrecisionTests.Div, v3, v4);
        return v5;
    };

    /**
     * @constructor
     * @extends {glsBuiltinPrecisionTests.DerivedFunc}
     */
    glsBuiltinPrecisionTests.Cosh = function() {
        var sig = new glsBuiltinPrecisionTests.Signature('float', 'float');
        glsBuiltinPrecisionTests.DerivedFunc.call(this, sig);
    };

    setParentClass(glsBuiltinPrecisionTests.Cosh, glsBuiltinPrecisionTests.DerivedFunc);

    glsBuiltinPrecisionTests.Cosh.prototype.getName = function() {
        return 'cosh';
    };

    glsBuiltinPrecisionTests.Cosh.prototype.doExpand = function(ctx, args) {
        // (exp(x) + exp(-x)) / constant(2.0f)
        var x = args.a;
        var v0 = app(new glsBuiltinPrecisionTests.Exp(), x);
        var v1 = app(new glsBuiltinPrecisionTests.Negate(), x);
        var v2 = app(new glsBuiltinPrecisionTests.Exp(), v1);
        var v3 = app(new glsBuiltinPrecisionTests.Add(), v0, v2);
        var v4 = new glsBuiltinPrecisionTests.Constant(2);
        var v5 = new glsBuiltinPrecisionTests.Apply('float', new glsBuiltinPrecisionTests.Div, v3, v4);
        return v5;
    };

    /**
     * @constructor
     * @extends {glsBuiltinPrecisionTests.DerivedFunc}
     */
    glsBuiltinPrecisionTests.Tanh = function() {
        var sig = new glsBuiltinPrecisionTests.Signature('float', 'float');
        glsBuiltinPrecisionTests.DerivedFunc.call(this, sig);
    };

    setParentClass(glsBuiltinPrecisionTests.Tanh, glsBuiltinPrecisionTests.DerivedFunc);

    glsBuiltinPrecisionTests.Tanh.prototype.getName = function() {
        return 'tanh';
    };

    glsBuiltinPrecisionTests.Tanh.prototype.doExpand = function(ctx, args) {
        // sinh(x) / cosh(x)
        var x = args.a;
        var v0 = app(new glsBuiltinPrecisionTests.Sinh(), x);
        var v1 = app(new glsBuiltinPrecisionTests.Cosh(), x);
        var v2 = new glsBuiltinPrecisionTests.Apply('float', new glsBuiltinPrecisionTests.Div, v0, v1);
        return v2;
    };

    /**
     * @constructor
     * @extends {glsBuiltinPrecisionTests.DerivedFunc}
     */
    glsBuiltinPrecisionTests.ASinh = function() {
        var sig = new glsBuiltinPrecisionTests.Signature('float', 'float');
        glsBuiltinPrecisionTests.DerivedFunc.call(this, sig);
    };

    setParentClass(glsBuiltinPrecisionTests.ASinh, glsBuiltinPrecisionTests.DerivedFunc);

    glsBuiltinPrecisionTests.ASinh.prototype.getName = function() {
        return 'asinh';
    };

    glsBuiltinPrecisionTests.ASinh.prototype.doExpand = function(ctx, args) {
        // log(x + sqrt(x * x + constant(1.0f)))
        var x = args.a;
        var v0 = app(new glsBuiltinPrecisionTests.Mul(), x, x);
        var v1 = new glsBuiltinPrecisionTests.Constant(1);
        var v2 = app(new glsBuiltinPrecisionTests.Add(), v0, v1);
        var v3 = app(new glsBuiltinPrecisionTests.Sqrt(), v2);
        var v4 = app(new glsBuiltinPrecisionTests.Add(), x, v3);
        var v5 = app(new glsBuiltinPrecisionTests.Log(), v4);
        return v5;
    };

    /**
     * @constructor
     * @extends {glsBuiltinPrecisionTests.DerivedFunc}
     */
    glsBuiltinPrecisionTests.ACosh = function() {
        var sig = new glsBuiltinPrecisionTests.Signature('float', 'float');
        glsBuiltinPrecisionTests.DerivedFunc.call(this, sig);
    };

    setParentClass(glsBuiltinPrecisionTests.ACosh, glsBuiltinPrecisionTests.DerivedFunc);

    glsBuiltinPrecisionTests.ACosh.prototype.getName = function() {
        return 'acosh';
    };

    glsBuiltinPrecisionTests.ACosh.prototype.doExpand = function(ctx, args) {
        // log(x + sqrt((x + constant(1.0f)) * (x - constant(1.0f))))
        var x = args.a;
        var one = new glsBuiltinPrecisionTests.Constant(1);
        var v0 = app(new glsBuiltinPrecisionTests.Add(), x, one);
        var v1 = app(new glsBuiltinPrecisionTests.Sub(), x, one);
        var v2 = app(new glsBuiltinPrecisionTests.Mul(), v0, v1);
        var v3 = app(new glsBuiltinPrecisionTests.Sqrt(), v2);
        var v4 = app(new glsBuiltinPrecisionTests.Add(), x, v3);
        var v5 = app(new glsBuiltinPrecisionTests.Log(), v4);
        return v5;
    };

    /**
     * @constructor
     * @extends {glsBuiltinPrecisionTests.DerivedFunc}
     */
    glsBuiltinPrecisionTests.ATanh = function() {
        var sig = new glsBuiltinPrecisionTests.Signature('float', 'float');
        glsBuiltinPrecisionTests.DerivedFunc.call(this, sig);
    };

    setParentClass(glsBuiltinPrecisionTests.ATanh, glsBuiltinPrecisionTests.DerivedFunc);

    glsBuiltinPrecisionTests.ATanh.prototype.getName = function() {
        return 'atanh';
    };

    glsBuiltinPrecisionTests.ATanh.prototype.doExpand = function(ctx, args) {
        // constant(0.5f) * log((constant(1.0f) + x) / (constant(1.0f) - x))
        var x = args.a;
        var one = new glsBuiltinPrecisionTests.Constant(1);
        var half = new glsBuiltinPrecisionTests.Constant(0.5);
        var v0 = app(new glsBuiltinPrecisionTests.Add(), one, x);
        var v1 = app(new glsBuiltinPrecisionTests.Sub(), one, x);
        var v2 = app(new glsBuiltinPrecisionTests.Div(), v0, v1);
        var v3 = app(new glsBuiltinPrecisionTests.Log(), v2);
        var v4 = app(new glsBuiltinPrecisionTests.Mul(), half, v3);
        return v4;
    };

    /**
     * @constructor
     * @extends {glsBuiltinPrecisionTests.DerivedFunc}
     */
    glsBuiltinPrecisionTests.Sqrt = function() {
        var sig = new glsBuiltinPrecisionTests.Signature('float', 'float');
        glsBuiltinPrecisionTests.DerivedFunc.call(this, sig);
    };

    setParentClass(glsBuiltinPrecisionTests.Sqrt, glsBuiltinPrecisionTests.DerivedFunc);

    glsBuiltinPrecisionTests.Sqrt.prototype.getName = function() {
        return 'sqrt';
    };

    glsBuiltinPrecisionTests.Sqrt.prototype.doExpand = function(ctx, args) {
        // constant(1.0f) / app<InverseSqrt>(x)
        var x = args.a;
        var one = new glsBuiltinPrecisionTests.Constant(1);
        var v0 = app(new glsBuiltinPrecisionTests.InverseSqrt(), x);
        var v1 = app(new glsBuiltinPrecisionTests.Div(), one, v0);
        return v1;
    };

    /**
     * @constructor
     * @extends {glsBuiltinPrecisionTests.DerivedFunc}
     */
    glsBuiltinPrecisionTests.Fract = function() {
        var sig = new glsBuiltinPrecisionTests.Signature('float', 'float');
        glsBuiltinPrecisionTests.DerivedFunc.call(this, sig);
    };

    setParentClass(glsBuiltinPrecisionTests.Fract, glsBuiltinPrecisionTests.DerivedFunc);

    glsBuiltinPrecisionTests.Fract.prototype.getName = function() {
        return 'fract';
    };

    glsBuiltinPrecisionTests.Fract.prototype.doExpand = function(ctx, args) {
        // x - floor(x)
        var x = args.a;
        var v0 = app(new glsBuiltinPrecisionTests.Floor(), x);
        var v1 = app(new glsBuiltinPrecisionTests.Sub(), x, v0);
        return v1;
    };

    /**
     * @constructor
     * @extends {glsBuiltinPrecisionTests.DerivedFunc}
     */
    glsBuiltinPrecisionTests.Mod = function() {
        var sig = new glsBuiltinPrecisionTests.Signature('float', 'float', 'float');
        glsBuiltinPrecisionTests.DerivedFunc.call(this, sig);
    };

    setParentClass(glsBuiltinPrecisionTests.Mod, glsBuiltinPrecisionTests.DerivedFunc);

    glsBuiltinPrecisionTests.Mod.prototype.getName = function() {
        return 'mod';
    };

    glsBuiltinPrecisionTests.Mod.prototype.doExpand = function(ctx, args) {
        // x - y * floor(x/y)
        var x = args.a;
        var y = args.b;
        var v0 = app(new glsBuiltinPrecisionTests.Div(), x, y);
        var v1 = app(new glsBuiltinPrecisionTests.Floor(), v0);
        var v2 = app(new glsBuiltinPrecisionTests.Mul(), y, v1);
        var v3 = app(new glsBuiltinPrecisionTests.Sub(), x, v2);
        return v3;
    };

    /**
     * @constructor
     * @extends {glsBuiltinPrecisionTests.PrimitiveFunc}
     */
    glsBuiltinPrecisionTests.Modf = function() {
        var sig = new glsBuiltinPrecisionTests.Signature('float', 'float', 'float');
        glsBuiltinPrecisionTests.PrimitiveFunc.call(this, sig);
    };
    setParentClass(glsBuiltinPrecisionTests.Modf, glsBuiltinPrecisionTests.PrimitiveFunc);

    glsBuiltinPrecisionTests.Modf.prototype.getName = function() {
        return 'modf';
    };

    glsBuiltinPrecisionTests.Modf.prototype.doApply = function(ctx, iargs, variablenames) {
        var intPart;
        var func1 = function(x) {
            intPart = Math.trunc(x);
            return x - intPart;
        };
        var func2 = function(x) {
            return Math.trunc(x);
        };

        var fracIV = tcuInterval.applyMonotone1p(func1, iargs.a);
        var wholeIV = tcuInterval.applyMonotone1p(func2, iargs.a);

        if (!iargs.a.isFinite()) {
            // Behavior on modf(Inf) not well-defined, allow anything as a fractional part
            // See Khronos bug 13907
            fracIV.operatorOrAssignBinary(tcuInterval.NAN);
        }

        ctx.env.m_map[variablenames[1]] = wholeIV;
        return fracIV;
    };

    glsBuiltinPrecisionTests.Modf.prototype.getOutParamIndex = function() {
        return 1;
    };

    /**
     * @constructor
     * @extends {glsBuiltinPrecisionTests.DerivedFunc}
     */
    glsBuiltinPrecisionTests.Mix = function() {
        var sig = new glsBuiltinPrecisionTests.Signature('float', 'float', 'float', 'float');
        glsBuiltinPrecisionTests.DerivedFunc.call(this, sig);
    };

    setParentClass(glsBuiltinPrecisionTests.Mix, glsBuiltinPrecisionTests.DerivedFunc);

    glsBuiltinPrecisionTests.Mix.prototype.getName = function() {
        return 'mix';
    };

    glsBuiltinPrecisionTests.Mix.prototype.operation1 = function(ctx, args) {
        // (x * (constant(1.0f) - a)) + y * a
        var x = args.a;
        var y = args.b;
        var a = args.c;
        var one = new glsBuiltinPrecisionTests.Constant(1);
        var v0 = app(new glsBuiltinPrecisionTests.Sub(), one, a);
        var v1 = app(new glsBuiltinPrecisionTests.Mul(), x, v0);
        var v2 = app(new glsBuiltinPrecisionTests.Mul(), y, a);
        var v3 = app(new glsBuiltinPrecisionTests.Add(), v1, v2);
        return v3;
    };

    glsBuiltinPrecisionTests.Mix.prototype.operation2 = function(ctx, args) {
        // x + (y - x) * a
        var x = args.a;
        var y = args.b;
        var a = args.c;
        var v0 = app(new glsBuiltinPrecisionTests.Sub(), y, x);
        var v1 = app(new glsBuiltinPrecisionTests.Mul(), a, v0);
        var v2 = app(new glsBuiltinPrecisionTests.Add(), x, v1);
        return v2;
    };

    glsBuiltinPrecisionTests.Mix.prototype.doExpand = function(ctx, args){
        return app(new glsBuiltinPrecisionTests.Alternatives(this.Sig), this.operation1(ctx, args), this.operation2(ctx, args), new glsBuiltinPrecisionTests.Void(), new glsBuiltinPrecisionTests.Void());
    }

    /**
     * @constructor
     * @extends {glsBuiltinPrecisionTests.DerivedFunc}
     */
    glsBuiltinPrecisionTests.SmoothStep = function() {
        var sig = new glsBuiltinPrecisionTests.Signature('float', 'float', 'float', 'float');
        glsBuiltinPrecisionTests.DerivedFunc.call(this, sig);
    };

    setParentClass(glsBuiltinPrecisionTests.SmoothStep, glsBuiltinPrecisionTests.DerivedFunc);

    glsBuiltinPrecisionTests.SmoothStep.prototype.getName = function() {
        return 'smoothstep';
    };

    glsBuiltinPrecisionTests.SmoothStep.prototype.doExpand = function(ctx, args) {
        var edge0 = args.a;
        var edge1 = args.b;
        var x = args.c;
        var zero = new glsBuiltinPrecisionTests.Constant(0);
        var one = new glsBuiltinPrecisionTests.Constant(1);
        //clamp((x - edge0) / (edge1 - edge0), constant(0.0f), constant(1.0f));
        var v0 = app(new glsBuiltinPrecisionTests.Sub(), x, edge0);
        var v1 = app(new glsBuiltinPrecisionTests.Sub(), edge1, edge0);
        var v2 = app(new glsBuiltinPrecisionTests.Div(), v0, v1);
        var v3 = app(new glsBuiltinPrecisionTests.Clamp(), v2, zero, one);
        var t = glsBuiltinPrecisionTests.bindExpression('float', 't', ctx, v3);
        //(t * t * (constant(3.0f) - constant(2.0f) * t))
        var two = new glsBuiltinPrecisionTests.Constant(2);
        var three = new glsBuiltinPrecisionTests.Constant(3);
        var v4 = app(new glsBuiltinPrecisionTests.Mul(), v3, v3);
        var v5 = app(new glsBuiltinPrecisionTests.Mul(), two, v3);
        var v6 = app(new glsBuiltinPrecisionTests.Sub(), three, v5);
        var v7 = app(new glsBuiltinPrecisionTests.Mul(), v4, v6);
        return v7;
    };

    /**
     * @constructor
     * @extends {glsBuiltinPrecisionTests.DerivedFunc}
     */
    glsBuiltinPrecisionTests.Pow = function() {
        var sig = new glsBuiltinPrecisionTests.Signature('float', 'float', 'float');
        glsBuiltinPrecisionTests.DerivedFunc.call(this, sig);
    };

    setParentClass(glsBuiltinPrecisionTests.Pow, glsBuiltinPrecisionTests.DerivedFunc);

    glsBuiltinPrecisionTests.Pow.prototype.getName = function() {
        return 'pow';
    };

    glsBuiltinPrecisionTests.Pow.prototype.doExpand = function(ctx, args) {
        // exp2(y * log2(x))
        var x = args.a;
        var y = args.b;
        var v0 = app(new glsBuiltinPrecisionTests.Log2(), x);
        var v1 = app(new glsBuiltinPrecisionTests.Mul(), y, v0);
        var v2 = app(new glsBuiltinPrecisionTests.Exp2(), v1);
        return v2;
    };

    /**
     * @constructor
     * @extends {glsBuiltinPrecisionTests.CFloatFunc1}
     */
    glsBuiltinPrecisionTests.ExpFunc = function(name, func) {
        glsBuiltinPrecisionTests.CFloatFunc1.call(this, name, func);
    };

    setParentClass(glsBuiltinPrecisionTests.ExpFunc, glsBuiltinPrecisionTests.CFloatFunc1);

    glsBuiltinPrecisionTests.ExpFunc.prototype.getCodomain = function() {
        return tcuInterval.withNumbers(0, Infinity);
    };

    glsBuiltinPrecisionTests.ExpFunc.prototype.precision = function(ctx, ret, x) {
        switch (ctx.floatPrecision) {
            case gluShaderUtil.precision.PRECISION_HIGHP:
                return ctx.format.ulp(ret, 3.0 + 2.0 * Math.abs(x));
            case gluShaderUtil.precision.PRECISION_MEDIUMP:
                return ctx.format.ulp(ret, 2.0 + 2.0 * Math.abs(x));
            case gluShaderUtil.precision.PRECISION_LOWP:
                return ctx.format.ulp(ret, 2.0);
            default:
                throw new Error(!'Impossible');
        }
    };

   /**
     * @constructor
     * @extends {glsBuiltinPrecisionTests.ExpFunc}
     */
    glsBuiltinPrecisionTests.Exp = function() {
        glsBuiltinPrecisionTests.ExpFunc.call(this, 'exp', Math.exp);
    };

    setParentClass(glsBuiltinPrecisionTests.Exp, glsBuiltinPrecisionTests.ExpFunc);

   /**
     * @constructor
     * @extends {glsBuiltinPrecisionTests.ExpFunc}
     */
    glsBuiltinPrecisionTests.Exp2 = function() {
        /**
         * @param {number} x
         * @return {number}
         */
        var exp2 = function(x) {
            return Math.exp(x * Math.LN2);
        };
        glsBuiltinPrecisionTests.ExpFunc.call(this, 'exp2', exp2);
    };

    setParentClass(glsBuiltinPrecisionTests.Exp2, glsBuiltinPrecisionTests.ExpFunc);

    /**
     * @constructor
     * @extends {glsBuiltinPrecisionTests.CFloatFunc1}
     */
    glsBuiltinPrecisionTests.LogFunc = function(name, func) {
        glsBuiltinPrecisionTests.CFloatFunc1.call(this, name, func);
    };

    setParentClass(glsBuiltinPrecisionTests.LogFunc, glsBuiltinPrecisionTests.CFloatFunc1);

    glsBuiltinPrecisionTests.LogFunc.prototype.precision = function(ctx, ret, x) {
        if (x <= 0)
            return NaN;
        switch (ctx.floatPrecision) {
            case gluShaderUtil.precision.PRECISION_HIGHP:
                return (0.5 <= x && x <= 2.0) ? deMath.deLdExp(1.0, -21) : ctx.format.ulp(ret, 3.0);
            case gluShaderUtil.precision.PRECISION_MEDIUMP:
                return (0.5 <= x && x <= 2.0) ? deMath.deLdExp(1.0, -7) : ctx.format.ulp(ret, 2.0);
            case gluShaderUtil.precision.PRECISION_LOWP:
                return ctx.format.ulp(ret, 2.0);
            default:
                throw new Error(!'Impossible');
        }
    };

   /**
     * @constructor
     * @extends {glsBuiltinPrecisionTests.LogFunc}
     */
    glsBuiltinPrecisionTests.Log = function() {
        glsBuiltinPrecisionTests.LogFunc.call(this, 'log', Math.log);
    };

    setParentClass(glsBuiltinPrecisionTests.Log, glsBuiltinPrecisionTests.LogFunc);

   /**
     * @constructor
     * @extends {glsBuiltinPrecisionTests.LogFunc}
     */
    glsBuiltinPrecisionTests.Log2 = function() {
        glsBuiltinPrecisionTests.LogFunc.call(this, 'log2', Math.log2);
    };

    setParentClass(glsBuiltinPrecisionTests.Log2, glsBuiltinPrecisionTests.LogFunc);

    /**
     * @constructor
     * @extends {glsBuiltinPrecisionTests.CFloatFunc1}
     */
    glsBuiltinPrecisionTests.PreciseFunc1 = function(name, func) {
        glsBuiltinPrecisionTests.CFloatFunc1.call(this, name, func);
    };

    setParentClass(glsBuiltinPrecisionTests.PreciseFunc1, glsBuiltinPrecisionTests.CFloatFunc1);

    glsBuiltinPrecisionTests.PreciseFunc1.prototype.precision = function(ctx, ret, x) {
        return 0;
    };

    /**
     * @constructor
     * @extends {glsBuiltinPrecisionTests.PreciseFunc1}
     */
    glsBuiltinPrecisionTests.Abs = function() {
        glsBuiltinPrecisionTests.PreciseFunc1.call(this, 'abs', Math.abs);
    };
    setParentClass(glsBuiltinPrecisionTests.Abs, glsBuiltinPrecisionTests.PreciseFunc1);

    /**
     * @constructor
     * @extends {glsBuiltinPrecisionTests.PreciseFunc1}
     */
    glsBuiltinPrecisionTests.Sign = function() {
        glsBuiltinPrecisionTests.PreciseFunc1.call(this, 'sign', Math.sign);
    };
    setParentClass(glsBuiltinPrecisionTests.Sign, glsBuiltinPrecisionTests.PreciseFunc1);

    /**
     * @constructor
     * @extends {glsBuiltinPrecisionTests.PreciseFunc1}
     */
    glsBuiltinPrecisionTests.Floor = function() {
        glsBuiltinPrecisionTests.PreciseFunc1.call(this, 'floor', Math.floor);
    };
    setParentClass(glsBuiltinPrecisionTests.Floor, glsBuiltinPrecisionTests.PreciseFunc1);

    /**
     * @constructor
     * @extends {glsBuiltinPrecisionTests.PreciseFunc1}
     */
    glsBuiltinPrecisionTests.RoundEven = function() {
        glsBuiltinPrecisionTests.PreciseFunc1.call(this, 'roundEven', deMath.rint);
    };
    setParentClass(glsBuiltinPrecisionTests.RoundEven, glsBuiltinPrecisionTests.PreciseFunc1);

    /**
     * @constructor
     * @extends {glsBuiltinPrecisionTests.PreciseFunc1}
     */
    glsBuiltinPrecisionTests.Ceil = function() {
        glsBuiltinPrecisionTests.PreciseFunc1.call(this, 'ceil', Math.ceil);
    };
    setParentClass(glsBuiltinPrecisionTests.Ceil, glsBuiltinPrecisionTests.PreciseFunc1);

    /**
     * @constructor
     * @extends {glsBuiltinPrecisionTests.PreciseFunc1}
     */
    glsBuiltinPrecisionTests.Trunc = function() {
        glsBuiltinPrecisionTests.PreciseFunc1.call(this, 'trunc', Math.trunc);
    };
    setParentClass(glsBuiltinPrecisionTests.Trunc, glsBuiltinPrecisionTests.PreciseFunc1);

    /**
     * @constructor
     * @extends {glsBuiltinPrecisionTests.CFloatFunc2}
     */
    glsBuiltinPrecisionTests.PreciseFunc2 = function(name, func) {
        glsBuiltinPrecisionTests.CFloatFunc2.call(this, name, func);
    };

    setParentClass(glsBuiltinPrecisionTests.PreciseFunc2, glsBuiltinPrecisionTests.CFloatFunc2);

    glsBuiltinPrecisionTests.PreciseFunc2.prototype.precision = function(ctx, ret, x, y) {
        return 0;
    };

    /**
     * @constructor
     * @extends {glsBuiltinPrecisionTests.PreciseFunc2}
     */
    glsBuiltinPrecisionTests.Min = function() {
        glsBuiltinPrecisionTests.PreciseFunc2.call(this, 'min', Math.min);
    };
    setParentClass(glsBuiltinPrecisionTests.Min, glsBuiltinPrecisionTests.PreciseFunc2);

    /**
     * @constructor
     * @extends {glsBuiltinPrecisionTests.PreciseFunc2}
     */
    glsBuiltinPrecisionTests.Max = function() {
        glsBuiltinPrecisionTests.PreciseFunc2.call(this, 'max', Math.max);
    };
    setParentClass(glsBuiltinPrecisionTests.Max, glsBuiltinPrecisionTests.PreciseFunc2);

    /**
     * @constructor
     * @extends {glsBuiltinPrecisionTests.PreciseFunc2}
     */
    glsBuiltinPrecisionTests.Step = function() {
        /**
         * @param {number} edge
         * @param {number} x
         * return number
         */
        var step = function(edge, x) {
            return x < edge ? 0.0 : 1.0;
        };
        glsBuiltinPrecisionTests.PreciseFunc2.call(this, 'step', step);
    };
    setParentClass(glsBuiltinPrecisionTests.Step, glsBuiltinPrecisionTests.PreciseFunc2);

    /**
     * @constructor
     * @extends {glsBuiltinPrecisionTests.CFloatFunc1}
     */
    glsBuiltinPrecisionTests.TrigFunc = function(name, func, loEx, hiEx) {
        glsBuiltinPrecisionTests.CFloatFunc1.call(this, name, func);
        this.m_loExtremum = loEx;
        this.m_hiExtremum = hiEx;
    };

    setParentClass(glsBuiltinPrecisionTests.TrigFunc, glsBuiltinPrecisionTests.CFloatFunc1);

    glsBuiltinPrecisionTests.TrigFunc.prototype.innerExtrema = function(ctx, angle) {
        var lo = angle.lo();
        var hi = angle.hi();
        var loSlope = this.doGetSlope(lo);
        var hiSlope = this.doGetSlope(hi);

        // Detect the high and low values the function can take between the
        // interval endpoints.
        if (angle.length() >= 2.0 * Math.PI) {
            // The interval is longer than a full cycle, so it must get all possible values.
            return this.m_hiExtremum.operatorOrBinary(this.m_loExtremum);
        } else if (loSlope == 1 && hiSlope == -1) {
            // The slope can change from positive to negative only at the maximum value.
            return this.m_hiExtremum;
        } else if (loSlope == -1 && hiSlope == 1) {
            // The slope can change from negative to positive only at the maximum value.
            return this.m_loExtremum;
        } else if (loSlope == hiSlope &&
                 deMath.deSign(this.applyExact(hi) - this.applyExact(lo)) * loSlope == -1) {
            // The slope has changed twice between the endpoints, so both extrema are included.
            return this.m_hiExtremum.operatorOrBinary(this.m_loExtremum);
        }

        return new tcuInterval.Interval();
    };

    glsBuiltinPrecisionTests.TrigFunc.prototype.getCodomain = function() {
        // Ensure that result is always within [-1, 1], or NaN (for +-inf)
        var v = tcuInterval.withIntervals(new tcuInterval.Interval(-1), new tcuInterval.Interval(1));
        return v.operatorOrBinary(tcuInterval.NAN);
    };

    glsBuiltinPrecisionTests.TrigFunc.prototype.precision = function(ctx, ret, arg) {
        if (ctx.floatPrecision == gluShaderUtil.precision.PRECISION_HIGHP) {
            // Use precision from OpenCL fast relaxed math
            if (-Math.PI <= arg && arg <= Math.PI) {
                return deMath.deLdExp(1.0, -11);
            } else {
                // "larger otherwise", let's pick |x| * 2^-12 , which is slightly over
                // 2^-11 at x == pi.
                return deMath.deLdExp(Math.abs(arg), -12);
            }
        } else if (ctx.floatPrecision == gluShaderUtil.precision.PRECISION_MEDIUMP) {
            if (-Math.PI <= arg && arg <= Math.PI) {
                // from OpenCL half-float extension specification
                return ctx.format.ulp(ret, 2.0);
            } else {
                // |x| * 2^-10 , slightly larger than 2 ULP at x == pi
                return deMath.deLdExp(Math.abs(arg), -10);
            }
        } else {
            // from OpenCL half-float extension specification
            return ctx.format.ulp(ret, 2.0);
        }
    };

    /**
     * @param {number} angle
     * @return number
     */
    glsBuiltinPrecisionTests.TrigFunc.prototype.doGetSlope = function(angle) {
        throw new Error('Virtual function. Please override.');
    };

    /**
     * @constructor
     * @extends {glsBuiltinPrecisionTests.TrigFunc}
     */
    glsBuiltinPrecisionTests.Sin = function() {
        glsBuiltinPrecisionTests.TrigFunc.call(this, 'sin', Math.sin, new tcuInterval.Interval(-1), new tcuInterval.Interval(1));
    };

    setParentClass(glsBuiltinPrecisionTests.Sin, glsBuiltinPrecisionTests.TrigFunc);

    glsBuiltinPrecisionTests.Sin.prototype.doGetSlope = function(angle) {
        return deMath.deSign(Math.cos(angle));
    };

    /**
     * @constructor
     * @extends {glsBuiltinPrecisionTests.TrigFunc}
     */
    glsBuiltinPrecisionTests.Cos = function() {
        glsBuiltinPrecisionTests.TrigFunc.call(this, 'cos', Math.cos, new tcuInterval.Interval(-1), new tcuInterval.Interval(1));
    };

    setParentClass(glsBuiltinPrecisionTests.Cos, glsBuiltinPrecisionTests.TrigFunc);

    glsBuiltinPrecisionTests.Cos.prototype.doGetSlope = function(angle) {
        return -deMath.deSign(Math.sin(angle));
    };

    /**
     * @constructor
     * @extends {glsBuiltinPrecisionTests.DerivedFunc}
     */
    glsBuiltinPrecisionTests.Tan = function() {
        var sig = new glsBuiltinPrecisionTests.Signature('float', 'float');
        glsBuiltinPrecisionTests.DerivedFunc.call(this, sig);
    };

    setParentClass(glsBuiltinPrecisionTests.Tan, glsBuiltinPrecisionTests.DerivedFunc);

    glsBuiltinPrecisionTests.Tan.prototype.getName = function() {
        return 'tan';
    };

    glsBuiltinPrecisionTests.Tan.prototype.doExpand = function(ctx, args) {
        //  sin(x) * (constant(1.0f) / cos(x)
        var x = args.a;
        var sin = app(new glsBuiltinPrecisionTests.Sin(), x);
        var cos = app(new glsBuiltinPrecisionTests.Cos(), x);
        var expr = app(new glsBuiltinPrecisionTests.Div(),
                                                      new glsBuiltinPrecisionTests.Constant(1),
                                                      cos);

        expr = new glsBuiltinPrecisionTests.Apply('float', new glsBuiltinPrecisionTests.Mul(),
                                                    sin,
                                                    expr);
        return expr;
    };

    /**
     * @constructor
     * @extends {glsBuiltinPrecisionTests.CFloatFunc1}
     */
    glsBuiltinPrecisionTests.ASin = function() {
        glsBuiltinPrecisionTests.CFloatFunc1.call(this, 'asin', Math.asin);
    };

    setParentClass(glsBuiltinPrecisionTests.ASin, glsBuiltinPrecisionTests.CFloatFunc1);

    glsBuiltinPrecisionTests.ASin.prototype.precision = function(ctx, ret, x) {
        if (!deMath.deInBounds32(x, -1.0, 1.0))
            return NaN;

        if (ctx.floatPrecision == gluShaderUtil.precision.PRECISION_HIGHP) {
            // Absolute error of 2^-11
            return deMath.deLdExp(1.0, -11);
        } else {
            // Absolute error of 2^-8
            return deMath.deLdExp(1.0, -8);
        }
    };

    /**
     * @constructor
     * @extends {glsBuiltinPrecisionTests.CFloatFunc1}
     */
    glsBuiltinPrecisionTests.ArcTrigFunc = function(name, func, precisionULPs, domain, coddomain) {
        glsBuiltinPrecisionTests.CFloatFunc1.call(this, name, func);
        this.m_precision = precisionULPs;
        this.m_domain = domain;
        this.m_codomain = coddomain;
    };

    setParentClass(glsBuiltinPrecisionTests.ArcTrigFunc, glsBuiltinPrecisionTests.CFloatFunc1);

    glsBuiltinPrecisionTests.ArcTrigFunc.prototype.precision = function(ctx, ret, x) {
        if (!this.m_domain.contains(new tcuInterval.Interval(x)))
            return NaN;

        if (ctx.floatPrecision == gluShaderUtil.precision.PRECISION_HIGHP) {
            // Use OpenCL's precision
            return ctx.format.ulp(ret, this.m_precision);
        } else {
            // Use OpenCL half-float spec
            return ctx.format.ulp(ret, 2.0);
        }
    };

    /**
     * @constructor
     * @extends {glsBuiltinPrecisionTests.ArcTrigFunc}
     */
    glsBuiltinPrecisionTests.ACos = function() {
        glsBuiltinPrecisionTests.ArcTrigFunc.call(this, 'acos', Math.acos, 4096.0,
                                                tcuInterval.withNumbers(-1, 1),
                                                tcuInterval.withNumbers(0, Math.PI));
    };

    setParentClass(glsBuiltinPrecisionTests.ACos, glsBuiltinPrecisionTests.ArcTrigFunc);

    /**
     * @constructor
     * @extends {glsBuiltinPrecisionTests.ArcTrigFunc}
     */
    glsBuiltinPrecisionTests.ATan = function() {
        glsBuiltinPrecisionTests.ArcTrigFunc.call(this, 'atan', Math.atan, 4096.0,
                                                tcuInterval.unbounded(),
                                                tcuInterval.withNumbers(-Math.PI * 0.5, Math.PI * 0.5));
    };

    setParentClass(glsBuiltinPrecisionTests.ATan, glsBuiltinPrecisionTests.ArcTrigFunc);

    /**
     * @constructor
     * @extends {glsBuiltinPrecisionTests.CFloatFunc2}
     */
    glsBuiltinPrecisionTests.ATan2 = function() {
        glsBuiltinPrecisionTests.CFloatFunc2.call(this, 'atan', Math.atan2);
    };

    setParentClass(glsBuiltinPrecisionTests.ATan2, glsBuiltinPrecisionTests.CFloatFunc2);

    glsBuiltinPrecisionTests.ATan2.prototype.innerExtrema = function(ctx, xi, yi) {
        var ret = new tcuInterval.Interval();

        if (yi.contains(tcuInterval.ZERO)) {
            if (xi.contains(tcuInterval.ZERO))
                ret.operatorOrAssignBinary(tcuInterval.NAN);
            if (xi.intersects(tcuInterval.withNumbers(-Infinity, 0)))
                ret.operatorOrAssignBinary(tcuInterval.withNumbers(-Math.PI, Math.PI));
        }

        if (ctx.format.hasInf() != tcuFloatFormat.YesNoMaybe.YES && (!yi.isFinite() || !xi.isFinite())) {
            // Infinities may not be supported, allow anything, including NaN
            ret.operatorOrAssignBinary(tcuInterval.NAN);
        }

        return ret;
    };

    glsBuiltinPrecisionTests.ATan2.prototype.precision = function(ctx, ret, x, y) {
        if (ctx.floatPrecision == gluShaderUtil.precision.PRECISION_HIGHP)
            return ctx.format.ulp(ret, 4096.0);
        else
            return ctx.format.ulp(ret, 2.0);
    };

    /**
     * @constructor
     * @param {number} size
     * @extends {glsBuiltinPrecisionTests.DerivedFunc}
     */
    glsBuiltinPrecisionTests.DeterminantBase = function(size) {
        var sig = new glsBuiltinPrecisionTests.Signature('float', 'mat' + size);
        glsBuiltinPrecisionTests.DerivedFunc.call(this, sig);
    };

    setParentClass(glsBuiltinPrecisionTests.DeterminantBase, glsBuiltinPrecisionTests.DerivedFunc);

    glsBuiltinPrecisionTests.DeterminantBase.prototype.getName = function() {
        return 'determinant';
    };

   /**
     * @constructor
     * @extends {glsBuiltinPrecisionTests.DeterminantBase}
     */
    glsBuiltinPrecisionTests.Determinant = function() {
        // TODO: Support sizes 3 and 4
        this.size = 2;
        glsBuiltinPrecisionTests.DeterminantBase.call(this, this.size);
    };

    setParentClass(glsBuiltinPrecisionTests.Determinant, glsBuiltinPrecisionTests.DeterminantBase);

    glsBuiltinPrecisionTests.Determinant.prototype.doExpand = function(ctx, args) {
        //  mat[0][0] * mat[1][1] - mat[1][0] * mat[0][1]
        var elem0_0 = new glsBuiltinPrecisionTests.MatrixVariable(args.a, 0, 0);
        var elem0_1 = new glsBuiltinPrecisionTests.MatrixVariable(args.a, 0, 1);
        var elem1_0 = new glsBuiltinPrecisionTests.MatrixVariable(args.a, 1, 0);
        var elem1_1 = new glsBuiltinPrecisionTests.MatrixVariable(args.a, 1, 1);

        var val0 = app(new glsBuiltinPrecisionTests.Mul(), elem0_0, elem1_1);
        var val1 = app(new glsBuiltinPrecisionTests.Mul(), elem0_1, elem1_0);
        return new glsBuiltinPrecisionTests.Apply('float', new glsBuiltinPrecisionTests.Sub(), val0, val1);
    };

    /**
     * @constructor
     * @extends {glsBuiltinPrecisionTests.DerivedFunc}
     */
    glsBuiltinPrecisionTests.Inverse = function() {
        this.size = 2;
        var name = 'mat' + this.size;
        var sig = new glsBuiltinPrecisionTests.Signature(name, name);
        glsBuiltinPrecisionTests.DerivedFunc.call(this, sig);
    };

    setParentClass(glsBuiltinPrecisionTests.Inverse, glsBuiltinPrecisionTests.DerivedFunc);

    glsBuiltinPrecisionTests.Inverse.prototype.getName = function() {
        return 'inverse';
    };

    glsBuiltinPrecisionTests.Inverse.prototype.doExpand = function(ctx, args) {
        var mat = args.a;
        var v0 = app(new glsBuiltinPrecisionTests.Determinant(), mat);
        var det = glsBuiltinPrecisionTests.bindExpression('float', 'det', ctx, v0);

        var elem0_0 = new glsBuiltinPrecisionTests.MatrixVariable(args.a, 0, 0);
        var elem0_1 = new glsBuiltinPrecisionTests.MatrixVariable(args.a, 0, 1);
        var elem1_0 = new glsBuiltinPrecisionTests.MatrixVariable(args.a, 1, 0);
        var elem1_1 = new glsBuiltinPrecisionTests.MatrixVariable(args.a, 1, 1);

        var result0_0 = app(new glsBuiltinPrecisionTests.Div(), elem1_1, det);
        var result0_1 = app(new glsBuiltinPrecisionTests.Div(), elem0_1, det);
        result0_1 = app(new glsBuiltinPrecisionTests.Negate(), result0_1);
        var result1_0 = app(new glsBuiltinPrecisionTests.Div(), elem1_0, det);
        result1_0 = app(new glsBuiltinPrecisionTests.Negate(), result1_0);
        var result1_1 = app(new glsBuiltinPrecisionTests.Div(), elem0_0, det);

        var col0 = app(new glsBuiltinPrecisionTests.GenVec(this.size, true), result0_0, result1_0);
        var col1 = app(new glsBuiltinPrecisionTests.GenVec(this.size, true), result0_1, result1_1);
        var ret = app(new glsBuiltinPrecisionTests.GenMat(this.size, this.size), col0, col1);

        return ret;
    };

    /**
     * @param {glsBuiltinPrecisionTests.PrecisionTestContext} ctx
     * @param {glsBuiltinPrecisionTests.CaseFactory} factory
     * @return {tcuTestCase.DeqpTest}
     */
    glsBuiltinPrecisionTests.createFuncGroup = function(ctx, factory) {
        /** @type {tcuTestCase.DeqpTest} */ var group = tcuTestCase.newTest(factory.getName(), factory.getDesc());

        for (var precNdx in gluShaderUtil.precision) {
            /** @type {gluShaderUtil.precision} */ var precision = gluShaderUtil.precision[precNdx];
            /** @type {string} */ var precName = gluShaderUtil.getPrecisionName(precision);
            /** @type {tcuFloatFormat.FloatFormat} */ var fmt = ctx.formats[precision];
            /** @type {tcuFloatFormat.FloatFormat} */ var highpFmt = ctx.formats[gluShaderUtil.precision.PRECISION_HIGHP];

            for (var shaderNdx in ctx.shaderTypes) {
                /** @type {gluShaderProgram.shaderType} */ var shaderType = ctx.shaderTypes[shaderNdx];
                /** @type {string} */ var shaderName = gluShaderProgram.getShaderTypeName(shaderType);
                /** @type {string} */ var name = precName + '_' + shaderName;
                /** @type {glsBuiltinPrecisionTests.Context} */ var caseCtx = new glsBuiltinPrecisionTests.Context(name, fmt, highpFmt,
                                                 precision, shaderType, ctx.numRandoms);

                group.addChild(factory.createCase(caseCtx));
            }
        }

        return group;
    };

    /**
     * @param {glsBuiltinPrecisionTests.CaseFactories} cases
     * @param {Array<gluShaderProgram.shaderType>} shaderTypes
     * @param {tcuTestCase.DeqpTest} dstGroup
     */
    glsBuiltinPrecisionTests.addBuiltinPrecisionTests = function(cases, shaderTypes, dstGroup) {
        /** @type {tcuFloatFormat.FloatFormat} */ var highp = new tcuFloatFormat.FloatFormat(-126, 127, 23, true,
                                                 tcuFloatFormat.YesNoMaybe.MAYBE, // subnormals
                                                 tcuFloatFormat.YesNoMaybe.YES, // infinities
                                                 tcuFloatFormat.YesNoMaybe.MAYBE); // NaN
        // \todo [2014-04-01 lauri] Check these once Khronos bug 11840 is resolved.
        /** @type {tcuFloatFormat.FloatFormat} */ var mediump = new tcuFloatFormat.FloatFormat(-13, 13, 9, false);
        // A fixed-point format is just a floating point format with a fixed
        // exponent and support for subnormals.
        /** @type {tcuFloatFormat.FloatFormat} */ var lowp = new tcuFloatFormat.FloatFormat(0, 0, 7, false, tcuFloatFormat.YesNoMaybe.YES);
        /** @type {glsBuiltinPrecisionTests.PrecisionTestContext} */ var ctx = new glsBuiltinPrecisionTests.PrecisionTestContext(highp, mediump, lowp,
                                                 shaderTypes, 16384);

        for (var ndx = 0; ndx < cases.getFactories().length; ++ndx)
            dstGroup.addChild(glsBuiltinPrecisionTests.createFuncGroup(ctx, cases.getFactories()[ndx]));
    };

    /**
     * @param {function(new:glsBuiltinPrecisionTests.Func)} F
     * @param {glsBuiltinPrecisionTests.CaseFactories} funcs
     * @param {string=} name
     */
    glsBuiltinPrecisionTests.addScalarFactory = function(F, funcs, name) {
        if (name === undefined)
            name = (new F()).getName();

        funcs.addFactory(new glsBuiltinPrecisionTests.GenFuncCaseFactory(glsBuiltinPrecisionTests.makeVectorizedFuncs(F), name));
    };

    /**
     * @param {function(new:glsBuiltinPrecisionTests.Func)} F
     */
    glsBuiltinPrecisionTests.createSimpleFuncCaseFactory = function(F) {
        return new glsBuiltinPrecisionTests.SimpleFuncCaseFactory(new F());
    };

    /**
     * @param {number} caseId test case Id
     * @return {glsBuiltinPrecisionTests.CaseFactories}
     */
    glsBuiltinPrecisionTests.createES3BuiltinCases = function(caseId) {
        /** @type {glsBuiltinPrecisionTests.CaseFactories} */ var funcs = new glsBuiltinPrecisionTests.BuiltinFuncs();

        switch (caseId) {
            case 0: glsBuiltinPrecisionTests.addScalarFactory(glsBuiltinPrecisionTests.Add, funcs); break;
            case 1: glsBuiltinPrecisionTests.addScalarFactory(glsBuiltinPrecisionTests.Sub, funcs); break;
            case 2: glsBuiltinPrecisionTests.addScalarFactory(glsBuiltinPrecisionTests.Mul, funcs); break;
            case 3: glsBuiltinPrecisionTests.addScalarFactory(glsBuiltinPrecisionTests.Div, funcs); break;
            case 4: glsBuiltinPrecisionTests.addScalarFactory(glsBuiltinPrecisionTests.Radians, funcs); break;
            case 5: glsBuiltinPrecisionTests.addScalarFactory(glsBuiltinPrecisionTests.Degrees, funcs); break;
            case 6: glsBuiltinPrecisionTests.addScalarFactory(glsBuiltinPrecisionTests.Sin, funcs); break;
            case 7: glsBuiltinPrecisionTests.addScalarFactory(glsBuiltinPrecisionTests.Cos, funcs); break;
            case 8: glsBuiltinPrecisionTests.addScalarFactory(glsBuiltinPrecisionTests.Tan, funcs); break;
            case 9: glsBuiltinPrecisionTests.addScalarFactory(glsBuiltinPrecisionTests.ASin, funcs); break;
            case 10: glsBuiltinPrecisionTests.addScalarFactory(glsBuiltinPrecisionTests.ACos, funcs); break;
            case 11: glsBuiltinPrecisionTests.addScalarFactory(glsBuiltinPrecisionTests.ATan, funcs); break;
            case 12: glsBuiltinPrecisionTests.addScalarFactory(glsBuiltinPrecisionTests.ATan2, funcs, 'atan2'); break;
            case 13: glsBuiltinPrecisionTests.addScalarFactory(glsBuiltinPrecisionTests.Sinh, funcs); break;
            case 14: glsBuiltinPrecisionTests.addScalarFactory(glsBuiltinPrecisionTests.Cosh, funcs); break;
            case 15: glsBuiltinPrecisionTests.addScalarFactory(glsBuiltinPrecisionTests.Tanh, funcs); break;
            case 16: glsBuiltinPrecisionTests.addScalarFactory(glsBuiltinPrecisionTests.ASinh, funcs); break;
            case 17: glsBuiltinPrecisionTests.addScalarFactory(glsBuiltinPrecisionTests.ACosh, funcs); break;
            case 18: glsBuiltinPrecisionTests.addScalarFactory(glsBuiltinPrecisionTests.ATanh, funcs); break;
            case 19: glsBuiltinPrecisionTests.addScalarFactory(glsBuiltinPrecisionTests.Pow, funcs); break;
            case 20: glsBuiltinPrecisionTests.addScalarFactory(glsBuiltinPrecisionTests.Exp, funcs); break;
            case 21: glsBuiltinPrecisionTests.addScalarFactory(glsBuiltinPrecisionTests.Exp2, funcs); break;
            case 22: glsBuiltinPrecisionTests.addScalarFactory(glsBuiltinPrecisionTests.Log, funcs); break;
            case 23: glsBuiltinPrecisionTests.addScalarFactory(glsBuiltinPrecisionTests.Log2, funcs); break;
            case 24: glsBuiltinPrecisionTests.addScalarFactory(glsBuiltinPrecisionTests.Sqrt, funcs); break;
            case 25: glsBuiltinPrecisionTests.addScalarFactory(glsBuiltinPrecisionTests.InverseSqrt, funcs); break;
            case 26: glsBuiltinPrecisionTests.addScalarFactory(glsBuiltinPrecisionTests.Abs, funcs); break;
            case 27: glsBuiltinPrecisionTests.addScalarFactory(glsBuiltinPrecisionTests.Sign, funcs); break;
            case 28: glsBuiltinPrecisionTests.addScalarFactory(glsBuiltinPrecisionTests.Floor, funcs); break;
            case 29: glsBuiltinPrecisionTests.addScalarFactory(glsBuiltinPrecisionTests.Trunc, funcs); break;
            case 30: glsBuiltinPrecisionTests.addScalarFactory(glsBuiltinPrecisionTests.Round, funcs); break;
            case 31: glsBuiltinPrecisionTests.addScalarFactory(glsBuiltinPrecisionTests.RoundEven, funcs); break;
            case 32: glsBuiltinPrecisionTests.addScalarFactory(glsBuiltinPrecisionTests.Ceil, funcs); break;
            case 33: glsBuiltinPrecisionTests.addScalarFactory(glsBuiltinPrecisionTests.Fract, funcs); break;
            case 34: glsBuiltinPrecisionTests.addScalarFactory(glsBuiltinPrecisionTests.Mod, funcs); break;
            case 35: funcs.addFactory(glsBuiltinPrecisionTests.createSimpleFuncCaseFactory(glsBuiltinPrecisionTests.Modf)); break;
            case 36: glsBuiltinPrecisionTests.addScalarFactory(glsBuiltinPrecisionTests.Min, funcs); break;
            case 37: glsBuiltinPrecisionTests.addScalarFactory(glsBuiltinPrecisionTests.Max, funcs); break;
            case 38: glsBuiltinPrecisionTests.addScalarFactory(glsBuiltinPrecisionTests.Mix, funcs); break;
            case 39: glsBuiltinPrecisionTests.addScalarFactory(glsBuiltinPrecisionTests.Step, funcs); break;
            case 40: glsBuiltinPrecisionTests.addScalarFactory(glsBuiltinPrecisionTests.SmoothStep, funcs); break;
            case 41: glsBuiltinPrecisionTests.addScalarFactory(glsBuiltinPrecisionTests.Clamp, funcs); break;
            case 42: funcs.addFactory(new glsBuiltinPrecisionTests.TemplateFuncCaseFactory(glsBuiltinPrecisionTests.Length)); break;
            case 43: funcs.addFactory(new glsBuiltinPrecisionTests.TemplateFuncCaseFactory(glsBuiltinPrecisionTests.Distance)); break;
            case 44: funcs.addFactory(new glsBuiltinPrecisionTests.TemplateFuncCaseFactory(glsBuiltinPrecisionTests.Dot)); break;
            case 45: funcs.addFactory(glsBuiltinPrecisionTests.createSimpleFuncCaseFactory(glsBuiltinPrecisionTests.Cross)); break;
            case 46: funcs.addFactory(new glsBuiltinPrecisionTests.TemplateFuncCaseFactory(glsBuiltinPrecisionTests.Normalize)); break;
            case 47: funcs.addFactory(new glsBuiltinPrecisionTests.TemplateFuncCaseFactory(glsBuiltinPrecisionTests.FaceForward)); break;
            case 48: funcs.addFactory(new glsBuiltinPrecisionTests.TemplateFuncCaseFactory(glsBuiltinPrecisionTests.Reflect)); break;
            case 49: funcs.addFactory(new glsBuiltinPrecisionTests.TemplateFuncCaseFactory(glsBuiltinPrecisionTests.Refract)); break;
            case 50: funcs.addFactory(new glsBuiltinPrecisionTests.MatrixFuncCaseFactory(glsBuiltinPrecisionTests.MatrixCompMult)); break;
            case 51: funcs.addFactory(new glsBuiltinPrecisionTests.MatrixFuncCaseFactory(glsBuiltinPrecisionTests.OuterProduct)); break;
            case 52: funcs.addFactory(new glsBuiltinPrecisionTests.MatrixFuncCaseFactory(glsBuiltinPrecisionTests.Transpose)); break;
            case 53: funcs.addFactory(new glsBuiltinPrecisionTests.SquareMatrixFuncCaseFactory(glsBuiltinPrecisionTests.Determinant)); break;
            case 54: funcs.addFactory(new glsBuiltinPrecisionTests.SquareMatrixFuncCaseFactory(glsBuiltinPrecisionTests.Inverse)); break;
            default: break;
        }

        return funcs;
    };

});
