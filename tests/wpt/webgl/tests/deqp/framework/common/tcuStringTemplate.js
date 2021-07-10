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
goog.provide('framework.common.tcuStringTemplate');

goog.scope(function() {

var tcuStringTemplate = framework.common.tcuStringTemplate;

tcuStringTemplate.escapeRegExp = function(string) {
    return string.replace(/([.*+?^=!:$ {}()|\[\]\/\\])/g, '\\$1');
};

tcuStringTemplate.specialize = function(str, params) {
    var dst = str;
    for (var key in params) {
        var value = params[key];
        var re = new RegExp(tcuStringTemplate.escapeRegExp('\$\{' + key + '\}'), 'g');
        dst = dst.replace(re, value);
    }
    return dst;
};

});
