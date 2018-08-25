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
goog.provide('modules.shared.glsStateQuery');

goog.scope(function() {
var glsStateQuery = modules.shared.glsStateQuery;

/**
 * Compare two objects. Objects must have the same type and contents.
 * If comparing numbers, allow some epsilon differences
 * @param {*} a
 * @param {*} b
 * return {boolean}
 */
glsStateQuery.compare = function(a, b) {
    /** @const */ var eps = 0.01;
    if (a === b)
        return true;

    if (typeof a === 'number' && typeof b === 'number')
        return Math.abs(a - b) < eps;

    //compare array-like parameters
    if (typeof a == 'object' && typeof b == 'object') {
        if (a.constructor !== b.constructor)
            return false;

        if ('length' in a && 'length' in b) {
            if (a.length !== b.length)
                return false;
            for (var i = 0; i < a.length; i++) {
                if (typeof a[i] === 'number' && typeof b[i] === 'number') {
                    if (Math.abs(a[i] - b[i]) >= eps)
                        return false;
                } else if (a[i] !== b[i])
                    return false;
            }
            return true;
        }

    }
    return false;
};

/**
 * Verify that WebGL state 'param' has the expected value
 * @param {number} param
 * @param {*} reference
 * @return {boolean}
 */
glsStateQuery.verify = function(param, reference) {
    var value = gl.getParameter(param);
    var result = glsStateQuery.compare(value, reference);
    if (!result) {
        bufferedLogToConsole('Result: ' + value + ' Expected: ' + reference);
    }
    return result;
};

/**
 * Verify that WebGL current vertex attrib has the expected value
 * @param {number} index
 * @param {*} reference
 * @return {boolean}
 */
glsStateQuery.verifyCurrentVertexAttrib = function(index, reference) {
    var value = gl.getVertexAttrib(index, gl.CURRENT_VERTEX_ATTRIB);
    var result = glsStateQuery.compare(value, reference);
    if (!result) {
        bufferedLogToConsole('Result: ' + value + ' Expected: ' + reference);
    }
    return result;
};

/**
 * Verify that WebGL vertex attrib attribute 'param' has the expected value
 * @param {number} index
 * @param {number} param
 * @param {*} reference
 * @return {boolean}
 */
glsStateQuery.verifyVertexAttrib = function(index, param, reference) {
    var value = (param == gl.VERTEX_ATTRIB_ARRAY_POINTER) ?
        gl.getVertexAttribOffset(index, param) :
        gl.getVertexAttrib(index, param);
    var result = glsStateQuery.compare(value, reference);
    if (!result) {
        bufferedLogToConsole('Result: ' + value + ' Expected: ' + reference);
    }
    return result;
};

/**
 * Verify that WebGL uniform has the expected value
 * @param {WebGLProgram} program
 * @param {WebGLUniformLocation} location
 * @param {*} reference
 * @return {boolean}
 */
glsStateQuery.verifyUniform = function(program, location, reference) {
    var value = gl.getUniform(program, location);
    var result = glsStateQuery.compare(value, reference);
    if (!result) {
        bufferedLogToConsole('Result: ' + value + ' Expected: ' + reference);
    }
    return result;
};

/**
 * Verify that WebGL shader state 'param' has the expected value
 * @param {WebGLShader} shader
 * @param {number} param
 * @param {*} reference
 * @return {boolean}
 */
glsStateQuery.verifyShader = function(shader, param, reference) {
    var value = gl.getShaderParameter(shader, param);
    var result = glsStateQuery.compare(value, reference);
    if (!result) {
        bufferedLogToConsole('Result: ' + value + ' Expected: ' + reference);
    }
    return result;
};

/**
 * Verify that WebGL program state 'param' has the expected value
 * @param {WebGLProgram} program
 * @param {number} param
 * @param {*} reference
 * @return {boolean}
 */
glsStateQuery.verifyProgram = function(program, param, reference) {
    var value = gl.getProgramParameter(program, param);
    var result = glsStateQuery.compare(value, reference);
    if (!result) {
        bufferedLogToConsole('Result: ' + value + ' Expected: ' + reference);
    }
    return result;
};

/**
 * Verify that WebGL sampler state 'param' has the expected value
 * @param {WebGLSampler} sampler
 * @param {number} param
 * @param {*} reference
 * @return {boolean}
 */
glsStateQuery.verifySampler = function(sampler, param, reference) {
    var value = gl.getSamplerParameter(sampler, param);
    var result = glsStateQuery.compare(value, reference);
    if (!result) {
        bufferedLogToConsole('Result: ' + value + ' Expected: ' + reference);
    }
    return result;
};

/**
 * Verify that WebGL texture state 'param' has the expected value
 * @param {number} target
 * @param {number} param
 * @param {*} reference
 * @return {boolean}
 */
glsStateQuery.verifyTexture = function(target, param, reference) {
    var value = gl.getTexParameter(target, param);
    var result = glsStateQuery.compare(value, reference);
    if (!result) {
        bufferedLogToConsole('Result: ' + value + ' Expected: ' + reference);
    }
    return result;
};

/**
 * Verify that WebGL state 'param' has one of the expected values
 * @param {number} param
 * @param {Array<*>} reference
 * return {boolean}
 */
glsStateQuery.verifyAnyOf = function(param, reference) {
    var value = gl.getParameter(param);
    for (var i = 0; i < reference.length; i++)
        if (glsStateQuery.compare(value, reference[i]))
            return true;
    bufferedLogToConsole('Result: ' + value + ' Expected one of: ' + reference);
    return false;
};

/**
 * Verify that WebGL state 'param' has the expected value
 * @param {number} param
 * @param {number|Array<number>} reference
 * @return {boolean}
 */
glsStateQuery.verifyGreaterOrEqual = function(param, reference) {
    var value = gl.getParameter(param);
    if (reference instanceof Array) {
        var v = /** @type {Array<number>} */ (value);
        if (v.length != reference.length) {
            bufferedLogToConsole('Result: ' + value + ' Expected >= : ' + reference);
            return false;
        }
        for (var i = 0; i < reference.length; i++)
            if (v[i] < reference[i]) {
                bufferedLogToConsole('Result: ' + value + ' Expected >= : ' + reference);
                return false;
            }
        return true;
    }
    var n = /** @type {number} */ (value);
    if (n < reference) {
        bufferedLogToConsole('Result: ' + value + ' Expected >= : ' + reference);
        return false;
    }
    return true;
};

/**
 * Verify that WebGL state 'param' has the expected value
 * @param {number} param
 * @param {number|Array<number>} reference
 * @return {boolean}
 */
glsStateQuery.verifyLessOrEqual = function(param, reference) {
    var value = gl.getParameter(param);
    if (reference instanceof Array) {
        var v = /** @type {Array<number>} */ (value);
        if (v.length != reference.length) {
            bufferedLogToConsole('Result: ' + value + ' Expected <= : ' + reference);
            return false;
        }
        for (var i = 0; i > reference.length; i++)
            if (v[i] < reference[i]) {
            bufferedLogToConsole('Result: ' + value + ' Expected <= : ' + reference);
            return false;
            }
        return true;
    }
    var n = /** @type {number} */ (value);
    if (n > reference) {
        bufferedLogToConsole('Result: ' + value + ' Expected <= : ' + reference);
        return false;
    }
    return true;
};

/**
 * Verify that WebGL state 'param' has the expected value (value & mask == reference)
 * @param {number} param
 * @param {number} reference
 * @param {number} mask
 * @return {boolean}
 */
glsStateQuery.verifyMasked = function(param, reference, mask) {
    var value = /** @type {number} */ (gl.getParameter(param));
    if ((value & mask) !== reference) {
        bufferedLogToConsole('Result: ' + value + ' Expected: ' + reference + 'Mask: 0x' + mask.toString(16));
        return false;
    }
    return true;
};

/**
 * Verify that WebGL fbo attachment 'param' has the expected value
 * @param {number} fbo
 * @param {number} attachment
 * @param {number} param
 * @param {*} reference
 * @return {boolean}
 */
glsStateQuery.verifyAttachment = function(fbo, attachment, param, reference) {
    var value = gl.getFramebufferAttachmentParameter(fbo, attachment, param);
    var result = glsStateQuery.compare(value, reference);
    if (!result) {
        bufferedLogToConsole('Result: ' + value + ' Expected: ' + reference);
    }
    return result;
};

/**
 * Verify that WebGL fbo color attachment 'param' has the expected value
 * @param {number} fbo
 * @param {number} param
 * @param {*} reference
 * @return {boolean}
 */
glsStateQuery.verifyColorAttachment = function(fbo, param, reference) {
    return glsStateQuery.verifyAttachment(fbo, gl.COLOR_ATTACHMENT0, param, reference);
};

/**
 * Verify that WebGL rbo attribute 'param' has the expected value
 * @param {number} param
 * @param {*} reference
 * @return {boolean}
 */
glsStateQuery.verifyRenderbuffer = function(param, reference) {
    var value = gl.getRenderbufferParameter(gl.RENDERBUFFER, param);
    var result = glsStateQuery.compare(value, reference);
    if (!result) {
        bufferedLogToConsole('Result: ' + value + ' Expected: ' + reference);
    }
    return result;
};

/**
 * Verify that WebGL active uniform block's attribute 'param' has the expected value
 * @param {WebGLProgram} program
 * @param {number} index
 * @param {number} param
 * @param {*} reference
 * @return {boolean}
 */
glsStateQuery.verifyActiveUniformBlock = function(program, index, param, reference) {
    var value = gl.getActiveUniformBlockParameter(program, index, param);
    var result = glsStateQuery.compare(value, reference);
    if (!result) {
        bufferedLogToConsole('Result: ' + value + ' Expected: ' + reference);
    }
    return result;
};

/**
 * @param  {number} param
 * @param  {Array<number>} reference
 * @param  {Array<boolean>} enableRef
 * @return  {boolean}
 */

glsStateQuery.verifyMask = function(param, reference, enableRef) {
    var intVector4 = /** @type {Array<number>} */ (gl.getParameter(param));

    if ((enableRef[0] && (intVector4[0] != reference[0])) ||
        (enableRef[1] && (intVector4[1] != reference[1])) ||
        (enableRef[2] && (intVector4[2] != reference[2])) ||
        (enableRef[3] && (intVector4[3] != reference[3])))
    {
        bufferedLogToConsole("// ERROR: expected " +
                             (enableRef[0] ? "" : "(") + reference[0] + (enableRef[0] ? "" : ")") + ", " +
                             (enableRef[1] ? "" : "(") + reference[1] + (enableRef[1] ? "" : ")") + ", " +
                             (enableRef[2] ? "" : "(") + reference[2] + (enableRef[2] ? "" : ")") + ", " +
                             (enableRef[3] ? "" : "(") + reference[3] + (enableRef[3] ? "" : ")"));

        return false;
    }
    return true;
};

});
