/*
 * Copyright 2013 The Closure Compiler Authors
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
 * @fileoverview This file describes the externs API for V8-specific objects.
 * @externs
 */



/**
 * Stack frame elements in V8.
 * @constructor
 */
function CallSite() {}


/**
 * Returns the value of this.
 * @return {Object|undefined}
 */
CallSite.prototype.getThis = function() {};


/**
 * Returns the type of this as a string. This is the name of the function stored
 * in the constructor field of this, if available, otherwise the object's
 * [[Class]] internal property.
 * @return {string|undefined}
 */
CallSite.prototype.getTypeName = function() {};


/**
 * Returns the current function.
 * @return {!Function|undefined}
 */
CallSite.prototype.getFunction = function() {};


/**
 * Returns the name of the current function, typically its name property. If a
 * name property is not available an attempt will be made to try to infer a name
 * from the function's context.
 * @return {string|undefined}
 */
CallSite.prototype.getFunctionName = function() {};


/**
 * Returns the name of the property of this or one of its prototypes that holds
 * the current function.
 * @return {string|undefined}
 */
CallSite.prototype.getMethodName = function() {};


/**
 * If this function was defined in a script returns the name of the script
 * @return {string|undefined}
 */
CallSite.prototype.getFileName = function() {};


/**
 * If this function was defined in a script returns the current line number.
 * @return {number|undefined}
 */
CallSite.prototype.getLineNumber = function() {};


/**
 * If this function was defined in a script returns the current column number.
 * @return {number|undefined}
 */
CallSite.prototype.getColumnNumber = function() {};


/**
 * If this function was created using a call to eval, returns a CallSite object
 * representing the location where eval was called
 * @return {CallSite|undefined}
 */
CallSite.prototype.getEvalOrigin = function() {};


/**
 * Is this a toplevel invocation, that is, is this the global object?
 * @return {boolean}
 */
CallSite.prototype.isToplevel = function() {};


/**
 * Does this call take place in code defined by a call to eval?
 * @return {boolean}
 */
CallSite.prototype.isEval = function() {};


/**
 * Is this call in native V8 code?
 * @return {boolean}
 */
CallSite.prototype.isNative = function() {};


/**
 * Is this a constructor call?
 * @return {boolean}
 */
CallSite.prototype.isConstructor = function() {};
