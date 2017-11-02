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
 * @fileoverview Definitions for all the Flash Object JavaScript methods. This
 * file depends on w3c_dom2.js.
 * Created from
 * http://www.adobe.com/support/flash/publishexport/scriptingwithflash/scriptingwithflash_03.html
 *
 * @externs
 */


// Standard Methods.

/**
 * Call a Flash function exported by ExternalInterface.
 * @param {string} xmlString The XML string passed to Flash. The outer element
 *     should be {@code <invoke>}. A sample invocation string:
 *     {@code <invoke name="function_name" returntype="javascript">
 *     <string>test</string></invoke>}
 * @return {string} The serialized return value from Flash that you can eval.
 */
HTMLObjectElement.prototype.CallFunction = function(xmlString) {};

/**
 * Returns the value of the Flash variable specified by varName or null if the
 * variable does not exist.
 * @param {string} varName The variable name.
 * @return {string?} The variable value.
 */
HTMLObjectElement.prototype.GetVariable = function(varName) {};

/**
 * Activates the frame number specified by {@code frameNumber} in the current
 * movie.
 * @param {number} frameNumber A non-negative integer frame number.
 */
HTMLObjectElement.prototype.GotoFrame = function(frameNumber) {};

/**
 * @return {boolean} Whether the movie is currently playing.
 */
HTMLObjectElement.prototype.IsPlaying = function() {};

/**
 * Loads the movie identified by {@code url} to the layer specified by {@code
 * layerNumber}.
 * @param {number} layerNumber The layer number.
 * @param {string} url The movie URL.
 */
HTMLObjectElement.prototype.LoadMovie = function(layerNumber, url) {};

/**
 * Pans a zoomed-in movie to the coordinates specified by x and y. Use mode to
 * specify whether the values for x and y are pixels or a percent of the window.
 * When mode is 0, the coordinates are pixels; when mode is 1, the coordinates
 * are percent of the window.
 * @param {number} x The x-coordinate.
 * @param {number} y The y-coordinate.
 * @param {number} mode The mode.
 */
HTMLObjectElement.prototype.Pan = function(x, y, mode) {};

/**
 * @return {number} The percent of the Flash Player movie that has streamed
 *     into the browser so far; Possible values are from 0 to 100.
 */
HTMLObjectElement.prototype.PercentLoaded = function() {};

/**
 * Starts playing the movie.
 */
HTMLObjectElement.prototype.Play = function() {};

/**
 * Goes to the first frame.
 */
HTMLObjectElement.prototype.Rewind = function() {};

/**
 * Sets the value of the flash variable.
 * @param {string} variableName The variable name.
 * @param {string} value The value.
 */
HTMLObjectElement.prototype.SetVariable = function(variableName, value) {};

/**
 * Zooms in on a rectangular area of the movie. The units of the coordinates
 * are in twips (1440 units per inch).
 * @param {number} left The left coordinate.
 * @param {number} top The top coordinate.
 * @param {number} right The right coordinate.
 * @param {number} bottom The bottom coordinate.
 */
HTMLObjectElement.prototype.SetZoomRect = function(left, top, right, bottom) {};

/**
 * Stops playing the movie.
 */
HTMLObjectElement.prototype.StopPlay = function() {};

/**
 * @return {number} The total number of frames in the movie.
 */
HTMLObjectElement.prototype.TotalFrames = function() {};

/**
 * Zooms the view by a relative scale factor.
 * @param {number} percent The percentage scale factor, should be an integer.
 */
HTMLObjectElement.prototype.Zoom = function(percent) {};


// TellTarget Methods.

/**
 * Executes the action in the timeline specified by {@code target} in the
 * specified frame.
 * @param {string} target The timeline.
 * @param {number} frameNumber The frame number.
 */
HTMLObjectElement.prototype.TCallFrame = function(target, frameNumber) {};

/**
 * Executes the action in the timeline specified by {@code target} in the
 * specified frame.
 * @param {string} target The timeline.
 * @param {string} label The frame label.
 */
HTMLObjectElement.prototype.TCallLabel = function(target, label) {};

/**
 * Returns the number of the current frame for the specified timeline.
 * @param {string} target The timeline.
 * @return {number} The number of the current frame.
 */
HTMLObjectElement.prototype.TCurentFrame = function(target) {};

/**
 * Returns the label of the current frame for the specified timeline.
 * @param {string} target The timeline.
 * @return {string} The label of the current frame, empty string if no
 *     current frame.
 */
HTMLObjectElement.prototype.TCurrentLabel = function(target) {};

/**
 * Returns a string indicating the value of the property in the
 * specified timeline.
 * @param {string} target The timeline.
 * @param {number} property The integer corresponding to the desired property.
 * @return {string} The value of the property.
 */
HTMLObjectElement.prototype.TGetProperty = function(target, property) {};

/**
 * Returns a number indicating the value of the property in the specified
 * timeline.
 * @param {string} target The timeline.
 * @param {number} property The integer corresponding to the desired property.
 * @return {number} A number indicating the value of the property.
 */
HTMLObjectElement.prototype.TGetPropertyAsNumber = function(target, property) {};

/**
 * Goes to the specified frame number in the specified timeline.
 * @param {string} target The timeline.
 * @param {number} frameNumber The frame number.
 */
HTMLObjectElement.prototype.TGotoFrame = function(target, frameNumber) {};

/**
 * Goes to the specified frame label in the specified timeline.
 * @param {string} target The timeline.
 * @param {string} label The framelabel.
 */
HTMLObjectElement.prototype.TGotoLabel = function(target, label) {};

/**
 * Plays the specified timeline.
 * @param {number} target The timeline.
 */
HTMLObjectElement.prototype.TPlay = function(target) {};

/**
 * Sets the value of the property in the specified timeline.
 * @param {number} target The timeline.
 * @param {number} property The integer corresponding to the desired property.
 * @param {string|number} value The value.
 */
HTMLObjectElement.prototype.TSetProperty = function(target, property, value) {};

/**
 * Stops the specified timeline.
 * @param {number} target The timeline.
 */
HTMLObjectElement.prototype.TStopPlay = function(target) {};
