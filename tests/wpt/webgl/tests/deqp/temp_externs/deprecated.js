/*
 * Copyright 2008 The Closure Compiler Authors
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

/**
 * @fileoverview JavaScript Built-Ins that are not
 * part of any specifications but are
 * still needed in some project's build.
 * @externs
 *
 */

// Do we need an opera.js?
var opera;
Window.prototype.opera;
Window.prototype.opera.postError;

/** @constructor */ function XSLTProcessor() {}
/**
 * @param {*=} opt_text
 * @param {*=} opt_value
 * @param {*=} opt_defaultSelected
 * @param {*=} opt_selected
 * @constructor
 * @extends {Element}
 */
function Option(opt_text, opt_value, opt_defaultSelected, opt_selected) {}


// The "methods" object is a place to hang arbitrary external
// properties. It is a throwback to pre-typed days, and should
// not be used for any new definitions; it exists only to bridge
// the gap between the old way and the new way.
var methods = {};
