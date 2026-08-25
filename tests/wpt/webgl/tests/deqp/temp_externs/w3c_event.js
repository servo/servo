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
 * @fileoverview Definitions for W3C's event specification.
 *  The whole file has been fully type annotated.
 *  Created from
 *   http://www.w3.org/TR/DOM-Level-2-Events/ecma-script-binding.html
 *
 * @externs
 */


/**
 * @interface
 */
function EventTarget() {}

/**
 * @param {string} type
 * @param {EventListener|function(!Event):(boolean|undefined)} listener
 * @param {boolean} useCapture
 * @return {undefined}
 */
EventTarget.prototype.addEventListener = function(type, listener, useCapture)
    {};

/**
 * @param {string} type
 * @param {EventListener|function(!Event):(boolean|undefined)} listener
 * @param {boolean} useCapture
 * @return {undefined}
 */
EventTarget.prototype.removeEventListener = function(type, listener, useCapture)
    {};

/**
 * @param {!Event} evt
 * @return {boolean}
 */
EventTarget.prototype.dispatchEvent = function(evt) {};

/**
 * @interface
 */
function EventListener() {}

/**
 * @param {!Event} evt
 * @return {undefined}
 */
EventListener.prototype.handleEvent = function(evt) {};

// The EventInit interface and the parameters to the Event constructor are part
// of DOM Level 3 (suggested) and the DOM "Living Standard" (mandated). They are
// included here as externs cannot be redefined. The same applies to other
// *EventInit interfaces and *Event constructors throughout this file. See:
// http://www.w3.org/TR/DOM-Level-3-Events/#event-initializers
// http://dom.spec.whatwg.org/#constructing-events
// https://dvcs.w3.org/hg/d4e/raw-file/tip/source_respec.htm#event-constructors

/**
 * @typedef {{
 *   bubbles: (boolean|undefined),
 *   cancelable: (boolean|undefined)
 * }}
 */
var EventInit;

/**
 * @constructor
 * @param {string} type
 * @param {EventInit=} opt_eventInitDict
 */
function Event(type, opt_eventInitDict) {}

/**
 * @type {number}
 * @see http://www.w3.org/TR/DOM-Level-2-Events/ecma-script-binding.html
 */
Event.AT_TARGET;

/**
 * @type {number}
 * @see http://www.w3.org/TR/DOM-Level-2-Events/ecma-script-binding.html
 */
Event.BUBBLING_PHASE;

/**
 * @type {number}
 * @see http://www.w3.org/TR/DOM-Level-2-Events/ecma-script-binding.html
 */
Event.CAPTURING_PHASE;


/** @type {string} */
Event.prototype.type;

/** @type {EventTarget} */
Event.prototype.target;

/** @type {EventTarget} */
Event.prototype.currentTarget;

/** @type {number} */
Event.prototype.eventPhase;

/** @type {boolean} */
Event.prototype.bubbles;

/** @type {boolean} */
Event.prototype.cancelable;

/** @type {number} */
Event.prototype.timeStamp;

/**
 * Present for events spawned in browsers that support shadow dom.
 * @type {Array.<!Element>|undefined}
 */
Event.prototype.path;

/**
 * @return {undefined}
 */
Event.prototype.stopPropagation = function() {};

/**
 * @return {undefined}
 */
Event.prototype.preventDefault = function() {};

/**
 * @param {string} eventTypeArg
 * @param {boolean} canBubbleArg
 * @param {boolean} cancelableArg
 * @return {undefined}
 */
Event.prototype.initEvent = function(eventTypeArg, canBubbleArg, cancelableArg) {};

/**
 * @typedef {{
 *   bubbles: (boolean|undefined),
 *   cancelable: (boolean|undefined),
 *   detail: *
 * }}
 */
var CustomEventInit;

/**
 * @constructor
 * @extends {Event}
 * @param {string} type
 * @param {CustomEventInit=} opt_eventInitDict
 * @see http://www.w3.org/TR/DOM-Level-3-Events/#interface-CustomEvent
 */
function CustomEvent(type, opt_eventInitDict) {}

/**
 * @param {string} eventType
 * @param {boolean} bubbles
 * @param {boolean} cancelable
 * @param {*} detail
 */
CustomEvent.prototype.initCustomEvent = function(
    eventType, bubbles, cancelable, detail) {};

/**
 * @type {*}
 */
CustomEvent.prototype.detail;

/**
 * @interface
 */
function DocumentEvent() {}

/**
 * @param {string} eventType
 * @return {!Event}
 */
DocumentEvent.prototype.createEvent = function(eventType) {};

/**
 * @typedef {{
 *   bubbles: (boolean|undefined),
 *   cancelable: (boolean|undefined),
 *   view: (Window|undefined),
 *   detail: (number|undefined)
 * }}
 */
var UIEventInit;

/**
 * @constructor
 * @extends {Event}
 * @param {string} type
 * @param {UIEventInit=} opt_eventInitDict
 */
function UIEvent(type, opt_eventInitDict) {}

/** @type {number} */
UIEvent.prototype.detail;

/**
 * @param {string} typeArg
 * @param {boolean} canBubbleArg
 * @param {boolean} cancelableArg
 * @param {Window} viewArg
 * @param {number} detailArg
 * @return {undefined}
 */
UIEvent.prototype.initUIEvent = function(typeArg, canBubbleArg, cancelableArg,
    viewArg, detailArg) {};

/**
 * @typedef {{
 *   bubbles: (boolean|undefined),
 *   cancelable: (boolean|undefined),
 *   view: (Window|undefined),
 *   detail: (number|undefined),
 *   screenX: (number|undefined),
 *   screenY: (number|undefined),
 *   clientX: (number|undefined),
 *   clientY: (number|undefined),
 *   ctrlKey: (boolean|undefined),
 *   shiftKey: (boolean|undefined),
 *   altKey: (boolean|undefined),
 *   metaKey: (boolean|undefined),
 *   button: (number|undefined),
 *   buttons: (number|undefined),
 *   relatedTarget: (EventTarget|undefined)
 * }}
 */
var MouseEventInit;

/**
 * @constructor
 * @extends {UIEvent}
 * @param {string} type
 * @param {MouseEventInit=} opt_eventInitDict
 */
function MouseEvent(type, opt_eventInitDict) {}

/** @type {number} */
MouseEvent.prototype.screenX;

/** @type {number} */
MouseEvent.prototype.screenY;

/** @type {number} */
MouseEvent.prototype.clientX;

/** @type {number} */
MouseEvent.prototype.clientY;

/** @type {boolean} */
MouseEvent.prototype.ctrlKey;

/** @type {boolean} */
MouseEvent.prototype.shiftKey;

/** @type {boolean} */
MouseEvent.prototype.altKey;

/** @type {boolean} */
MouseEvent.prototype.metaKey;

/** @type {number} */
MouseEvent.prototype.button;

/** @type {EventTarget} */
MouseEvent.prototype.relatedTarget;


/**
 * @constructor
 * @extends {Event}
 */
function MutationEvent() {}

/** @type {Node} */
MutationEvent.prototype.relatedNode;

/** @type {string} */
MutationEvent.prototype.prevValue;

/** @type {string} */
MutationEvent.prototype.newValue;

/** @type {string} */
MutationEvent.prototype.attrName;

/** @type {number} */
MutationEvent.prototype.attrChange;

/**
 * @param {string} typeArg
 * @param {boolean} canBubbleArg
 * @param {boolean} cancelableArg
 * @param {Node} relatedNodeArg
 * @param {string} prevValueArg
 * @param {string} newValueArg
 * @param {string} attrNameArg
 * @param {number} attrChangeArg
 * @return {undefined}
 */
MutationEvent.prototype.initMutationEvent = function(typeArg, canBubbleArg, cancelableArg, relatedNodeArg, prevValueArg, newValueArg, attrNameArg, attrChangeArg) {};


// DOM3
/**
 * @typedef {{
 *   bubbles: (boolean|undefined),
 *   cancelable: (boolean|undefined),
 *   view: (Window|undefined),
 *   detail: (number|undefined),
 *   char: (string|undefined),
 *   key: (string|undefined),
 *   code: (string|undefined),
 *   location: (number|undefined),
 *   ctrlKey: (boolean|undefined),
 *   shiftKey: (boolean|undefined),
 *   altKey: (boolean|undefined),
 *   metaKey: (boolean|undefined),
 *   repeat: (boolean|undefined),
 *   locale: (string|undefined)
 * }}
 */
var KeyboardEventInit;

/**
 * @constructor
 * @extends {UIEvent}
 * @param {string} type
 * @param {KeyboardEventInit=} opt_eventInitDict
 */
function KeyboardEvent(type, opt_eventInitDict) {}

/** @type {string} */
KeyboardEvent.prototype.keyIdentifier;

/** @type {boolean} */
KeyboardEvent.prototype.ctrlKey;

/** @type {boolean} */
KeyboardEvent.prototype.shiftKey;

/** @type {boolean} */
KeyboardEvent.prototype.altKey;

/** @type {boolean} */
KeyboardEvent.prototype.metaKey;

/**
 * @param {string} keyIdentifierArg
 * @return {boolean}
 */
KeyboardEvent.prototype.getModifierState = function(keyIdentifierArg) {};
