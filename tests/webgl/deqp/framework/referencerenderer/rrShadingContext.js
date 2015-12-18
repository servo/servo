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
goog.provide('framework.referencerenderer.rrShadingContext');
goog.require('framework.referencerenderer.rrFragmentOperations');
goog.require('framework.delibs.debase.deMath');
goog.require('framework.referencerenderer.rrDefs');
goog.require('framework.referencerenderer.rrFragmentOperations');
goog.require('framework.referencerenderer.rrGenericVector');

goog.scope(function() {

    var rrShadingContext = framework.referencerenderer.rrShadingContext;
    var deMath = framework.delibs.debase.deMath;
    var rrDefs = framework.referencerenderer.rrDefs;
    var rrFragmentOperations = framework.referencerenderer.rrFragmentOperations;
    var rrGenericVector = framework.referencerenderer.rrGenericVector;

    var DE_ASSERT = function(x) {
        if (!x)
            throw new Error('Assert failed');
    };

    /**
     * Fragment shading context
     *
     * Contains per-primitive information used in fragment shading
     * @constructor
     * @param {Array<Array<number>>} varying0 (GenericVec4*)
     * @param {Array<Array<number>>} varying1 (GenericVec4*)
     * @param {Array<Array<number>>} varying2 (GenericVec4*)
     */
    rrShadingContext.FragmentShadingContext = function(varying0, varying1, varying2) {
        /** @type {Array<Array<Array<number>>>} */ this.varyings = [varying0, varying1, varying2]; //!< Vertex shader outputs. Pointer will be NULL if there is no such vertex.
        this.m_width = 0xFFFFFFFF;
        this.m_height = 0xFFFFFFFF;
    };

    /**
     * @param {number} width
     * @param {number} height
     */
    rrShadingContext.FragmentShadingContext.prototype.setSize = function(width, height) {
        this.m_width = width;
        this.m_height = height;
    };

    rrShadingContext.FragmentShadingContext.prototype.getWidth = function() {
        return this.m_width;
    };

    rrShadingContext.FragmentShadingContext.prototype.getHeight = function() {
        return this.m_height;
    };

    // Read Varying

    /**
     * @param {rrFragmentOperations.Fragment} packet
     * @param {rrShadingContext.FragmentShadingContext} context
     * @param {number} varyingLoc
     * @return {Array<number>} (Vector<T, 4>)
     */
    rrShadingContext.readTriangleVarying = function(packet, context, varyingLoc) {
        var result = deMath.scale(
            context.varyings[0][varyingLoc],
            packet.barycentric[0]
        );

        if (context.varyings[1])
            result = deMath.add(result, deMath.scale(
                context.varyings[1][varyingLoc],
                packet.barycentric[1]
            ));

        if (context.varyings[2])
            result = deMath.add(result, deMath.scale(
                context.varyings[2][varyingLoc],
                packet.barycentric[2]
            ));

        return result;
    };

    /**
     * @param {rrFragmentOperations.Fragment} packet
     * @param {rrShadingContext.FragmentShadingContext} context
     * @param {number} varyingLoc
     * @return {Array<number>} (Vector<T, 4>)
     */
    rrShadingContext.readVarying = function(packet, context, varyingLoc) {
        return rrShadingContext.readTriangleVarying(packet, context, varyingLoc);
    };

});
