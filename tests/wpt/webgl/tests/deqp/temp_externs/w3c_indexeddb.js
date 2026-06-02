/*
 * Copyright 2011 The Closure Compiler Authors.
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
 * @fileoverview Definitions for W3C's IndexedDB API. In Chrome all the
 * IndexedDB classes are prefixed with 'webkit'. In order to access constants
 * and static methods of these classes they must be duplicated with the
 * prefix here.
 * @see http://www.w3.org/TR/IndexedDB/
 *
 * @externs
 * @author guido.tapia@picnet.com.au (Guido Tapia)
 */

/** @type {IDBFactory} */
Window.prototype.moz_indexedDB;

/** @type {IDBFactory} */
Window.prototype.mozIndexedDB;

/** @type {IDBFactory} */
Window.prototype.webkitIndexedDB;

/** @type {IDBFactory} */
Window.prototype.msIndexedDB;

/** @type {IDBFactory} */
Window.prototype.indexedDB;

/**
 * @constructor
 * @see http://www.w3.org/TR/IndexedDB/#idl-def-IDBFactory
 */
function IDBFactory() {}

/**
 * @param {string} name The name of the database to open.
 * @param {number=} opt_version The version at which to open the database.
 * @return {!IDBOpenDBRequest} The IDBRequest object.
 */
IDBFactory.prototype.open = function(name, opt_version) {};

/**
 * @param {string} name The name of the database to delete.
 * @return {!IDBOpenDBRequest} The IDBRequest object.
 */
IDBFactory.prototype.deleteDatabase = function(name) {};

/**
 * @constructor
 * @see http://www.w3.org/TR/IndexedDB/#idl-def-IDBDatabaseException
 */
function IDBDatabaseException() {}

/**
 * @constructor
 * @extends {IDBDatabaseException}
 * @see http://www.w3.org/TR/IndexedDB/#idl-def-IDBDatabaseException
 */
function webkitIDBDatabaseException() {}

/**
 * @const
 * @type {number}
 */
IDBDatabaseException.UNKNOWN_ERR;

/**
 * @const
 * @type {number}
 */
webkitIDBDatabaseException.UNKNOWN_ERR;

/**
 * @const
 * @type {number}
 */
IDBDatabaseException.NON_TRANSIENT_ERR;

/**
 * @const
 * @type {number}
 */
webkitIDBDatabaseException.NON_TRANSIENT_ERR;

/**
 * @const
 * @type {number}
 */
IDBDatabaseException.NOT_FOUND_ERR;

/**
 * @const
 * @type {number}
 */
webkitIDBDatabaseException.NOT_FOUND_ERR;

/**
 * @const
 * @type {number}
 */
IDBDatabaseException.CONSTRAINT_ERR;

/**
 * @const
 * @type {number}
 */
webkitIDBDatabaseException.CONSTRAINT_ERR;

/**
 * @const
 * @type {number}
 */
IDBDatabaseException.DATA_ERR;

/**
 * @const
 * @type {number}
 */
webkitIDBDatabaseException.DATA_ERR;

/**
 * @const
 * @type {number}
 */
IDBDatabaseException.NOT_ALLOWED_ERR;

/**
 * @const
 * @type {number}
 */
webkitIDBDatabaseException.NOT_ALLOWED_ERR;

/**
 * @const
 * @type {number}
 */
IDBDatabaseException.TRANSACTION_INACTIVE_ERR;

/**
 * @const
 * @type {number}
 */
webkitIDBDatabaseException.TRANSACTION_INACTIVE_ERR;

/**
 * @const
 * @type {number}
 */
IDBDatabaseException.ABORT_ERR;

/**
 * @const
 * @type {number}
 */
webkitIDBDatabaseException.ABORT_ERR;

/**
 * @const
 * @type {number}
 */
IDBDatabaseException.READ_ONLY_ERR;

/**
 * @const
 * @type {number}
 */
webkitIDBDatabaseException.READ_ONLY_ERR;

/**
 * @const
 * @type {number}
 */
IDBDatabaseException.TIMEOUT_ERR;

/**
 * @const
 * @type {number}
 */
webkitIDBDatabaseException.TIMEOUT_ERR;

/**
 * @const
 * @type {number}
 */
IDBDatabaseException.QUOTA_ERR;

/**
 * @const
 * @type {number}
 */
webkitIDBDatabaseException.QUOTA_ERR;

/**
 * @const
 * @type {number}
 */
IDBDatabaseException.prototype.code;

/**
 * @const
 * @type {number}
 */
webkitIDBDatabaseException.prototype.code;

/**
 * @const
 * @type {string}
 */
IDBDatabaseException.prototype.message;

/**
 * @const
 * @type {string}
 */
webkitIDBDatabaseException.prototype.message;

/**
 * @constructor
 * @implements {EventTarget}
 * @see http://www.w3.org/TR/IndexedDB/#idl-def-IDBRequest
 */
function IDBRequest() {}

/**
 * @param {boolean=} opt_useCapture
 * @override
 */
IDBRequest.prototype.addEventListener =
    function(type, listener, opt_useCapture) {};

/**
 * @param {boolean=} opt_useCapture
 * @override
 */
IDBRequest.prototype.removeEventListener =
    function(type, listener, opt_useCapture) {};

/** @override */
IDBRequest.prototype.dispatchEvent = function(evt) {};

/**
 * @constructor
 * @extends {IDBRequest}
 * @see http://www.w3.org/TR/IndexedDB/#idl-def-IDBRequest
 */
function webkitIDBRequest() {}

/**
 * @type {number}
 * @const
 */
IDBRequest.LOADING;

/**
 * @type {number}
 * @const
 */
webkitIDBRequest.LOADING;

/**
 * @type {number}
 * @const
 */
IDBRequest.DONE;

/**
 * @type {number}
 * @const
 */
webkitIDBRequest.DONE;

/** @type {number} */
IDBRequest.prototype.readyState; // readonly

/** @type {function(!Event)} */
IDBRequest.prototype.onsuccess = function(e) {};

/** @type {function(!Event)} */
IDBRequest.prototype.onerror = function(e) {};

/** @type {*} */
IDBRequest.prototype.result;  // readonly

/**
 * @type {number}
 * @deprecated Use "error"
 */
IDBRequest.prototype.errorCode;  // readonly


/** @type {!DOMError} */
IDBRequest.prototype.error; // readonly

/** @type {Object} */
IDBRequest.prototype.source; // readonly

/** @type {IDBTransaction} */
IDBRequest.prototype.transaction; // readonly

/**
 * @constructor
 * @extends {IDBRequest}
 * @see http://www.w3.org/TR/IndexedDB/#idl-def-IDBOpenDBRequest
 */
function IDBOpenDBRequest() {}

/**
 * @type {function(!IDBVersionChangeEvent)}
 */
IDBOpenDBRequest.prototype.onblocked = function(e) {};

/**
 * @type {function(!IDBVersionChangeEvent)}
 */
IDBOpenDBRequest.prototype.onupgradeneeded = function(e) {};

/**
 * @constructor
 * @implements {EventTarget}
 * @see http://www.w3.org/TR/IndexedDB/#idl-def-IDBDatabase
 */
function IDBDatabase() {}

/**
 * @type {string}
 * @const
 */
IDBDatabase.prototype.name;

/**
 * @type {string}
 * @const
 */
IDBDatabase.prototype.description;

/**
 * @type {string}
 * @const
 */
IDBDatabase.prototype.version;

/**
 * @type {DOMStringList}
 * @const
 */
IDBDatabase.prototype.objectStoreNames;

/**
 * @param {string} name The name of the object store.
 * @param {Object=} opt_parameters Parameters to be passed
 *     creating the object store.
 * @return {!IDBObjectStore} The created/open object store.
 */
IDBDatabase.prototype.createObjectStore =
    function(name, opt_parameters)  {};

/**
 * @param {string} name The name of the object store to remove.
 */
IDBDatabase.prototype.deleteObjectStore = function(name) {};

/**
 * @param {string} version The new version of the database.
 * @return {!IDBRequest} The IDBRequest object.
 */
IDBDatabase.prototype.setVersion = function(version) {};

/**
 * @param {Array.<string>} storeNames The stores to open in this transaction.
 * @param {(number|string)=} mode The mode for opening the object stores.
 * @return {!IDBTransaction} The IDBRequest object.
 */
IDBDatabase.prototype.transaction = function(storeNames, mode) {};

/**
 * Closes the database connection.
 */
IDBDatabase.prototype.close = function() {};

/**
 * @type {Function}
 */
IDBDatabase.prototype.onabort = function() {};

/**
 * @type {Function}
 */
IDBDatabase.prototype.onerror = function() {};

/**
 * @type {Function}
 */
IDBDatabase.prototype.onversionchange = function() {};

/**
 * @param {boolean=} opt_useCapture
 * @override
 */
IDBDatabase.prototype.addEventListener =
    function(type, listener, opt_useCapture) {};

/**
 * @param {boolean=} opt_useCapture
 * @override
 */
IDBDatabase.prototype.removeEventListener =
    function(type, listener, opt_useCapture) {};

/** @override */
IDBDatabase.prototype.dispatchEvent = function(evt) {};

/**
 * Typedef for valid key types according to the w3 specification. Note that this
 * is slightly wider than what is actually allowed, as all Array elements must
 * have a valid key type.
 * @see http://www.w3.org/TR/IndexedDB/#key-construct
 * @typedef {number|string|!Date|!Array.<?>}
 */
var IDBKeyType;

/**
 * @constructor
 * @see http://www.w3.org/TR/IndexedDB/#idl-def-IDBObjectStore
 */
function IDBObjectStore() {}

/**
 * @type {string}
 */
IDBObjectStore.prototype.name;

/**
 * @type {string}
 */
IDBObjectStore.prototype.keyPath;

/**
 * @type {DOMStringList}
 */
IDBObjectStore.prototype.indexNames;

/** @type {IDBTransaction} */
IDBObjectStore.prototype.transaction;

/** @type {boolean} */
IDBObjectStore.prototype.autoIncrement;

/**
 * @param {*} value The value to put into the object store.
 * @param {IDBKeyType=} key The key of this value.
 * @return {!IDBRequest} The IDBRequest object.
 */
IDBObjectStore.prototype.put = function(value, key) {};

/**
 * @param {*} value The value to add into the object store.
 * @param {IDBKeyType=} key The key of this value.
 * @return {!IDBRequest} The IDBRequest object.
 */
IDBObjectStore.prototype.add = function(value, key) {};

/**
 * @param {IDBKeyType} key The key of this value.
 * @return {!IDBRequest} The IDBRequest object.
 */
IDBObjectStore.prototype.delete = function(key) {};

/**
 * @param {IDBKeyType|!IDBKeyRange} key The key of the document to retrieve.
 * @return {!IDBRequest} The IDBRequest object.
 */
IDBObjectStore.prototype.get = function(key) {};

/**
 * @return {!IDBRequest} The IDBRequest object.
 */
IDBObjectStore.prototype.clear = function() {};

/**
 * @param {IDBKeyRange=} range The range of the cursor.
 * @param {(number|string)=} direction The direction of cursor enumeration.
 * @return {!IDBRequest} The IDBRequest object.
 */
IDBObjectStore.prototype.openCursor = function(range, direction) {};

/**
 * @param {string} name The name of the index.
 * @param {string|!Array.<string>} keyPath The path to the index key.
 * @param {Object=} opt_paramters Optional parameters
 *     for the created index.
 * @return {!IDBIndex} The IDBIndex object.
 */
IDBObjectStore.prototype.createIndex = function(name, keyPath, opt_paramters) {};

/**
 * @param {string} name The name of the index to retrieve.
 * @return {!IDBIndex} The IDBIndex object.
 */
IDBObjectStore.prototype.index = function(name) {};

/**
 * @param {string} indexName The name of the index to remove.
 */
IDBObjectStore.prototype.deleteIndex = function(indexName) {};

/**
 * @param {(IDBKeyType|IDBKeyRange)=} key The key of this value.
 * @return {!IDBRequest} The IDBRequest object.
 * @see http://www.w3.org/TR/IndexedDB/#widl-IDBObjectStore-count
 */
IDBObjectStore.prototype.count = function(key) {};

/**
 * @constructor
 * @see http://www.w3.org/TR/IndexedDB/#idl-def-IDBIndex
 */
function IDBIndex() {}

/**
 * @type {string}
 * @const
 */
IDBIndex.prototype.name;

/**
 * @type {!IDBObjectStore}
 * @const
 */
IDBIndex.prototype.objectStore;

/**
 * @type {string}
 * @const
 */
IDBIndex.prototype.keyPath;

/**
 * @type {boolean}
 * @const
 */
IDBIndex.prototype.unique;

/**
 * @param {IDBKeyRange=} range The range of the cursor.
 * @param {(number|string)=} direction The direction of cursor enumeration.
 * @return {!IDBRequest} The IDBRequest object.
 */
IDBIndex.prototype.openCursor = function(range, direction) {};

/**
 * @param {IDBKeyRange=} range The range of the cursor.
 * @param {(number|string)=} direction The direction of cursor enumeration.
 * @return {!IDBRequest} The IDBRequest object.
 */
IDBIndex.prototype.openKeyCursor = function(range, direction) {};

/**
 * @param {IDBKeyType|!IDBKeyRange} key The id of the object to retrieve.
 * @return {!IDBRequest} The IDBRequest object.
 */
IDBIndex.prototype.get = function(key) {};

/**
 * @param {IDBKeyType|!IDBKeyRange} key The id of the object to retrieve.
 * @return {!IDBRequest} The IDBRequest object.
 */
IDBIndex.prototype.getKey = function(key) {};

/**
 * @constructor
 * @see http://www.w3.org/TR/IndexedDB/#idl-def-IDBCursor
 */
function IDBCursor() {}

/**
 * @constructor
 * @extends {IDBCursor}
 * @see http://www.w3.org/TR/IndexedDB/#idl-def-IDBCursor
 */
function webkitIDBCursor() {}

/**
 * @const
 * @type {number}
 */
IDBCursor.NEXT;

/**
 * @const
 * @type {number}
 */
webkitIDBCursor.NEXT;

/**
 * @const
 * @type {number}
 */
IDBCursor.NEXT_NO_DUPLICATE;

/**
 * @const
 * @type {number}
 */
webkitIDBCursor.NEXT_NO_DUPLICATE;

/**
 * @const
 * @type {number}
 */
IDBCursor.PREV;

/**
 * @const
 * @type {number}
 */
webkitIDBCursor.PREV;

/**
 * @const
 * @type {number}
 */
IDBCursor.PREV_NO_DUPLICATE;

/**
 * @const
 * @type {number}
 */
webkitIDBCursor.PREV_NO_DUPLICATE;

/**
 * @type {*}
 * @const
 */
IDBCursor.prototype.source;

/**
 * @type {number}
 * @const
 */
IDBCursor.prototype.direction;

/**
 * @type {IDBKeyType}
 * @const
 */
IDBCursor.prototype.key;

/**
 * @type {number}
 * @const
 */
IDBCursor.prototype.primaryKey;

/**
 * @param {*} value The new value for the current object in the cursor.
 * @return {!IDBRequest} The IDBRequest object.
 */
IDBCursor.prototype.update = function(value) {};

/**
 * Note: Must be quoted to avoid parse error.
 * @param {IDBKeyType=} key Continue enumerating the cursor from the specified
 *     key (or next).
 */
IDBCursor.prototype.continue = function(key) {};

/**
 * @param {number} count Number of times to iterate the cursor.
 */
IDBCursor.prototype.advance = function(count) {};

/**
 * Note: Must be quoted to avoid parse error.
 * @return {!IDBRequest} The IDBRequest object.
 */
IDBCursor.prototype.delete = function() {};

/**
 * @constructor
 * @extends {IDBCursor}
 * @see http://www.w3.org/TR/IndexedDB/#idl-def-IDBCursorWithValue
 */
function IDBCursorWithValue() {}

/** @type {*} */
IDBCursorWithValue.prototype.value; // readonly

/**
 * @constructor
 * @see http://www.w3.org/TR/IndexedDB/#idl-def-IDBTransaction
 */
function IDBTransaction() {}

/**
 * @constructor
 * @extends {IDBTransaction}
 * @see http://www.w3.org/TR/IndexedDB/#idl-def-IDBTransaction
 */
function webkitIDBTransaction() {}

/**
 * @const
 * @type {number}
 */
IDBTransaction.READ_WRITE;

/**
 * @const
 * @type {number}
 */
webkitIDBTransaction.READ_WRITE;

/**
 * @const
 * @type {number}
 */
IDBTransaction.READ_ONLY;

/**
 * @const
 * @type {number}
 */
webkitIDBTransaction.READ_ONLY;

/**
 * @const
 * @type {number}
 */
IDBTransaction.VERSION_CHANGE;

/**
 * @const
 * @type {number}
 */
webkitIDBTransaction.VERSION_CHANGE;

/**
 * @type {number|string}
 * @const
 */
IDBTransaction.prototype.mode;

/**
 * @type {IDBDatabase}
 * @const
 */
IDBTransaction.prototype.db;

/**
 * @param {string} name The name of the object store to retrieve.
 * @return {!IDBObjectStore} The object store.
 */
IDBTransaction.prototype.objectStore = function(name) {};

/**
 * Aborts the transaction.
 */
IDBTransaction.prototype.abort = function() {};

/**
 * @type {Function}
 */
IDBTransaction.prototype.onabort = function() {};

/**
 * @type {Function}
 */
IDBTransaction.prototype.oncomplete = function() {};

/**
 * @type {Function}
 */
IDBTransaction.prototype.onerror = function() {};

/**
 * @constructor
 * @see http://www.w3.org/TR/IndexedDB/#idl-def-IDBKeyRange
 */
function IDBKeyRange() {}

/**
 * @constructor
 * @extends {IDBKeyRange}
 * @see http://www.w3.org/TR/IndexedDB/#idl-def-IDBKeyRange
 */
function webkitIDBKeyRange() {}

/**
 * @type {*}
 * @const
 */
IDBKeyRange.prototype.lower;

/**
 * @type {*}
 * @const
 */
IDBKeyRange.prototype.upper;

/**
 * @type {*}
 * @const
 */
IDBKeyRange.prototype.lowerOpen;

/**
 * @type {*}
 * @const
 */
IDBKeyRange.prototype.upperOpen;

/**
 * @param {IDBKeyType} value The single key value of this range.
 * @return {!IDBKeyRange} The key range.
 */
IDBKeyRange.only = function(value) {};

/**
 * @param {IDBKeyType} value The single key value of this range.
 * @return {!IDBKeyRange} The key range.
 */
webkitIDBKeyRange.only = function(value) {};

/**
 * @param {IDBKeyType} bound Creates a lower bound key range.
 * @param {boolean=} open Open the key range.
 * @return {!IDBKeyRange} The key range.
 */
IDBKeyRange.lowerBound = function(bound, open) {};

/**
 * @param {IDBKeyType} bound Creates a lower bound key range.
 * @param {boolean=} open Open the key range.
 * @return {!IDBKeyRange} The key range.
 */
webkitIDBKeyRange.lowerBound = function(bound, open) {};

/**
 * @param {IDBKeyType} bound Creates an upper bound key range.
 * @param {boolean=} open Open the key range.
 * @return {!IDBKeyRange} The key range.
 */
IDBKeyRange.upperBound = function(bound, open) {};

/**
 * @param {IDBKeyType} bound Creates an upper bound key range.
 * @param {boolean=} open Open the key range.
 * @return {!IDBKeyRange} The key range.
 */
webkitIDBKeyRange.upperBound = function(bound, open) {};

/**
 * @param {IDBKeyType} left The left bound value.
 * @param {IDBKeyType} right The right bound value.
 * @param {boolean=} openLeft Whether the left bound value should be excluded.
 * @param {boolean=} openRight Whether the right bound value should be excluded.
 * @return {!IDBKeyRange} The key range.
 */
IDBKeyRange.bound = function(left, right, openLeft, openRight) {};

/**
 * @param {IDBKeyType} left The left bound value.
 * @param {IDBKeyType} right The right bound value.
 * @param {boolean=} openLeft Whether the left bound value should be excluded.
 * @param {boolean=} openRight Whether the right bound value should be excluded.
 * @return {!IDBKeyRange} The key range.
 */
webkitIDBKeyRange.bound = function(left, right, openLeft, openRight) {};

/**
 * @constructor
 * @extends {Event}
 * @see http://www.w3.org/TR/IndexedDB/#idl-def-IDBVersionChangeEvent
 */
function IDBVersionChangeEvent() {}

/**
 * @type {number}
 * @const
 */
IDBVersionChangeEvent.prototype.oldVersion;

/**
 * @type {?number}
 * @const
 */
IDBVersionChangeEvent.prototype.newVersion;

/**
 * @constructor
 * @extends {IDBVersionChangeEvent}
 * @see http://www.w3.org/TR/IndexedDB/#idl-def-IDBVersionChangeEvent
 */
function webkitIDBVersionChangeEvent() {}

/**
 * @type {string}
 * @const
 */
webkitIDBVersionChangeEvent.prototype.version;
