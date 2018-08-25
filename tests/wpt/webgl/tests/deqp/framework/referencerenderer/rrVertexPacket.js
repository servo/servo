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
goog.provide('framework.referencerenderer.rrVertexPacket');
goog.require('framework.common.tcuTexture');
goog.require('framework.delibs.debase.deMath');

goog.scope(function() {

var rrVertexPacket = framework.referencerenderer.rrVertexPacket;
var tcuTexture = framework.common.tcuTexture;
var deMath = framework.delibs.debase.deMath;

    /**
     * rrVertexPacket.VertexPacket class. (Should only be created by rrVertexPacket.VertexPacketAllocator)
     * @constructor
     */
    rrVertexPacket.VertexPacket = function() {
        /** @type {number} */ this.instanceNdx;
        /** @type {number} */ this.vertexNdx;
        /** @type {goog.NumberArray} */ this.position; //!< Transformed position - must be written always.
        /** @type {number} */ this.pointSize; //!< Point size, required when rendering points.
        // /** @type {number} */ this.primitiveID; //!< Geometry shader output (Not used in webGL)
        /** @type {Array<goog.NumberArray>} */ this.outputs = [[0, 0, 0, 0]];
    };

    /**
     * rrVertexPacket.VertexPacketAllocator class
     * @constructor
     * @param {number} numberOfVertexOutputs
     */
    rrVertexPacket.VertexPacketAllocator = function(numberOfVertexOutputs) {
        /** @type {number} */ this.m_numberOfVertexOutputs = numberOfVertexOutputs;
        /** @type {Uint8Array} */ this.m_allocations;
        /** @type {Array<rrVertexPacket.VertexPacket>} */ this.m_singleAllocPool = [];
    };

    /**
     * @return {number}
     */
    rrVertexPacket.VertexPacketAllocator.prototype.getNumVertexOutputs = function() {
        return this.m_numberOfVertexOutputs;
    };

    /**
     * allocArray
     * @param {number} count
     * @return {Array<rrVertexPacket.VertexPacket>}
     */
    rrVertexPacket.VertexPacketAllocator.prototype.allocArray = function(count) {
        if (!count)
            return [];

        /** @type {number} */ var extraVaryings = (this.m_numberOfVertexOutputs == 0) ? (0) : (this.m_numberOfVertexOutputs - 1);
        // /** @type {number} TODO: Check what this size is used for */ var packetSize = sizeof(rrVertexPacket.VertexPacket) + extraVaryings * sizeof(GenericVec4);

        /** @type {Array<rrVertexPacket.VertexPacket>} */ var retVal = [];
        // /** @type {Uint8Array} TODO: same as above */ var ptr = new deInt8[packetSize * count]; // throws bad_alloc => ok

        //run ctors
        for (var i = 0; i < count; ++i)
            retVal.push(new rrVertexPacket.VertexPacket());

        /** TODO: same as previous - this.m_allocations.push_back(ptr); */

        return retVal;
    };

    /**
     * @return {rrVertexPacket.VertexPacket}
     */
    rrVertexPacket.VertexPacketAllocator.prototype.alloc = function() {
        /** @type {number} */ var poolSize = 8;

        if (this.m_singleAllocPool.length == 0)
            this.m_singleAllocPool = this.allocArray(poolSize);

        /** @type {rrVertexPacket.VertexPacket} */ var packet = this.m_singleAllocPool.pop();

        return packet;
    };

});
