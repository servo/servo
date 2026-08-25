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
goog.provide('framework.delibs.debase.deUtil');
goog.require('framework.delibs.debase.deMath');

goog.scope(function() {

    var deUtil = framework.delibs.debase.deUtil;
    var deMath = framework.delibs.debase.deMath;

    //! Get an element of an array with a specified size.
    /**
     * @param {Array} array
     * @param {number} offset
     * @return {*}
     */
    deUtil.getArrayElement = function(array, offset) {
        assertMsgOptions(deMath.deInBounds32(offset, 0, array.length), 'Array element out of bounds', false, true);
        return array[offset];
    };

    /**
     * clone - If you need to pass/assign an object by value, call this
     * @param {*} obj
     * @return {*}
     */
    deUtil.clone = function(obj) {
        if (obj == null || typeof(obj) != 'object')
            return obj;

        var temp = {};
        if (ArrayBuffer.isView(obj)) {
            temp = new obj.constructor(obj);
        } else if (obj instanceof Array) {
            temp = new Array(obj.length);
            for (var akey in obj)
                temp[akey] = deUtil.clone(obj[akey]);
        } else if (obj instanceof ArrayBuffer) {
            temp = new ArrayBuffer(obj.byteLength);
            var dst = new Uint8Array(temp);
            var src = new Uint8Array(obj);
            dst.set(src);
        } else {
            temp = Object.create(obj.constructor.prototype);
            temp.constructor = obj.constructor;
            for (var key in obj)
                temp[key] = deUtil.clone(obj[key]);
        }
        return temp;
    };

    /**
    * Add a push_unique function to Array. Will insert only if there is no equal element.
    * @template T
    * @param {Array<T>} array Any array
    * @param {T} object Any object
    */
    deUtil.dePushUniqueToArray = function(array, object) {
        //Simplest implementation
        for (var i = 0; i < array.length; i++) {
            if (object.equals !== undefined)
                if (object.equals(array[i]))
                    return undefined;
            else if (object === array[i])
                return undefined;
        }

        array.push(object);
    };

});
