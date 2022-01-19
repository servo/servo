// interface EventRecorder {
//    static void start();
//    static void stop();
//    static void clearRecords();
//    static sequence<EventRecord> getRecords();
//    static void configure(EventRecorderOptions options);
// };
// * getRecords
//   * returns an array of EventRecord objects; the array represents the sequence of events captured at anytime after the last clear()
//             call, between when the recorder was started and stopped (including multiple start/stop pairs)
// * configure
//   * sets options that should apply to the recorder. If the recorder has any existing records, than this API throws an exception.
// * start
//   * starts/un-pauses the recorder
// * stop
//   * stops/pauses the recorder
// * clear
//   * purges all recorded records

// ----------------------

// dictionary EventRecorderOptions {
//    sequence<SupportedEventTypes> mergeEventTypes;
//    ObjectNamedMap objectMap;
// };
// * mergeEventTypes
//   * a list of event types that should be consolidated into one record when all of the following conditions are true:
//     1) The events are of the same type and follow each other chronologically
//     2) The events' currentTarget is the same
//   * The default is an empty list (no event types are merged).
// * objectMap
//   * Sets up a series

// dictionary ObjectNamedMap {
//    //<keys will be 'targetTestID' names, with values of the objects which they label>
// };
//   * targetTestID = the string identifier that the associated target object should be known as (for purposes of unique identification. This
//                    need not be the same as the Node's id attribute if it has one. If no 'targetTestID' string mapping is provided via this
//                    map, but is encountered later when recording specific events, a generic targetTestID of 'UNKNOWN_OBJECT' is used.

// ----------------------

// dictionary EventRecord {
//    unsigned long chronologicalOrder;
//    unsigned long sequentialOccurrences;
//    sequence<EventRecord>? nestedEvents;
//    DOMString interfaceType;
//    EventRecordDetails event;
// };
// * chronologicalOrder
//   * Since some events may be dispatched re-entrantly (e.g., while existing events are being dispatched), and others may be merged
//     given the 'mergeEventTypes' option in the EventRecorder, this value is the actual chronological order that the event fired
// * sequentialOccurrences
//   * If this event was fired multiple times in a row (see the 'mergeEventTypes' option), this value is the count of occurrences.
//     A value of 1 means this was the only occurrence of this event (that no events were merged with it). A value greater than 1
//     indicates that the event occurred that many times in a row.
// * nestedEvents
//   * The holds all the events that were sequentially dispatched synchronously while the current event was still being dispatched
//     (e.g., between the time that this event listener was triggered and when it returned).
//   * Has the value null if no nested events were recorded during the invocation of this listener.
// * interfaceType
//   * The string indicating which Event object (or derived Event object type) the recorded event object instance is based on.
// * event
//   * Access to the recorded event properties for the event instance (not the actual event instance itself). A snapshot of the
//     enumerable properties of the event object instance at the moment the listener was first triggered.

// ----------------------

// dictionary EventRecordDetails {
//    //<recorded property names with their values for all enumerable properties of the event object instance>
// };
// * EventRecordDetails
//   * For records with 'sequentialOccurrences' > 1, only the first occurence is recorded (subsequent event details are dropped).
//   * Object reference values (e.g., event.target, event.currentTarget, etc.) are replaced with their mapped 'targetTestID' string.
//     If no 'targetTestID' string mapping is available for a particular object, the value 'UNKNOWN_OBJECT' is returned.

// ----------------------

// partial interface Node {
//    void addRecordedEventListener(SupportedEventTypes type, EventListener? handler, optional boolean capturePhase = false);
//    void removeRecordedEventListener(SupportedEventTypes type, EventListener? handler, optional boolean capturePhase = false);
// };
//
// enum SupportedEventTypes = {
//    "mousemove",
//    etc...
// };
// * addRecordedEventListener
//   * handler =      pass null if you want only a default recording of the event (and don't need any other special handling). Otherwise,
//                    the handler will be invoked normally as part of the event's dispatch.
//   * <other params> are the same as those defined on addEventListener/removeEventListenter APIs (see DOM4)
//   * Use this API *instead of* addEventListener to record your events for testing purposes.

(function EventRecorderScope(global) {
   "use strict";

   if (global.EventRecorder)
      return; // Already initialized.

   // WeakMap polyfill
   if (!global.WeakMap) {
      throw new Error("EventRecorder depends on WeakMap! Please polyfill for completeness to run in this user agent!");
   }

   // Globally applicable variables
   var allRecords = [];
   var recording = false;
   var rawOrder = 1;
   var mergeTypesTruthMap = {}; // format of { eventType: true, ... }
   var eventsInScope = []; // Tracks synchronous event dispatches
   var handlerMap = new WeakMap(); // Keeps original handlers (so that they can be used to un-register for events.

   // Find all Event Object Constructors on the global and add them to the map along with their name (sans 'Event')
   var eventConstructorsNameMap = new WeakMap(); // format of key: hostObject, value: alias to use.
   var regex = /[A-Z][A-Za-z0-9]+Event$/;
   Object.getOwnPropertyNames(global).forEach(function (propName) {
        if (regex.test(propName))
         eventConstructorsNameMap.set(global[propName], propName);
   });
   var knownObjectsMap = eventConstructorsNameMap;

   Object.defineProperty(global, "EventRecorder", {
      writable: true,
      configurable: true,
      value: Object.create(null, {
         start: {
            enumerable: true, configurable: true, writable: true, value: function start() { recording = true; }
         },
         stop: {
            enumerable: true, configurable: true, writable: true, value: function stop() { recording = false; }
         },
         clearRecords: {
            enumerable: true, configurable: true, writable: true, value: function clearRecords() {
               rawOrder = 1;
               allRecords = [];
            }
         },
         getRecords: {
            enumerable: true, configurable: true, writable: true, value: function getRecords() { return allRecords; }
         },
         checkRecords: {
            enumerable: true, configurable: true, writable: true, value: function checkRecords(expected) {
               if (expected.length < allRecords.length) {
                  return false;
               }
               var j = 0;
               for (var i = 0; i < expected.length; ++i) {
                  if (j >= allRecords.length) {
                     if (expected[i].optional) {
                        continue;
                     }
                     return false;
                  }
                  if (expected[i].type == allRecords[j].event.type && expected[i].target == allRecords[j].event.currentTarget) {
                     ++j;
                     continue;
                  }
                  if (expected[i].optional) {
                     continue;
                  }
                  return false;
               }
               return true;
            }
         },
         configure: {
            enumerable: true, configurable: true, writable: true, value: function configure(options) {
               if (allRecords.length > 0)
                  throw new Error("Wrong time to call me: EventRecorder.configure must only be called when no recorded events are present. Try 'clearRecords' first.");

               // Un-configure existing options by calling again with no options set...
               mergeTypesTruthMap = {};
               knownObjectsMap = eventConstructorsNameMap;

               if (!(options instanceof Object))
                  return;
               // Sanitize the passed object (tease-out getter functions)
               var sanitizedOptions = {};
               for (var x in options) {
                  sanitizedOptions[x] = options[x];
               }
               if (sanitizedOptions.mergeEventTypes && Array.isArray(sanitizedOptions.mergeEventTypes)) {
                  sanitizedOptions.mergeEventTypes.forEach(function (eventType) {
                     if (typeof eventType == "string")
                        mergeTypesTruthMap[eventType] = true;
                  });
               }
               if (sanitizedOptions.objectMap && (sanitizedOptions.objectMap instanceof Object)) {
                  for (var y in sanitizedOptions.objectMap) {
                     knownObjectsMap.set(sanitizedOptions.objectMap[y], y);
                  }
               }
            }
         },
         addEventListenersForNodes: {
            enumerable: true, configurable: true, writable: true, value: function addEventListenersForNodes(events, nodes, handler) {
               for (var i = 0; i < nodes.length; ++i) {
                  for (var j = 0; j < events.length; ++j) {
                     nodes[i].addRecordedEventListener(events[j], handler);
                  }
               }
            }
         }
      })
   });

   function EventRecord(rawEvent) {
      this.chronologicalOrder = rawOrder++;
      this.sequentialOccurrences = 1;
      this.nestedEvents = null; // potentially a []
      this.interfaceType = knownObjectsMap.get(rawEvent.constructor);
      if (!this.interfaceType) // In case (somehow) this event's constructor is not named something with an 'Event' suffix...
         this.interfaceType = rawEvent.constructor.toString();
      this.event = new CloneObjectLike(rawEvent);
   }

   // Only enumerable props including prototype-chain (non-recursive), w/no functions.
   function CloneObjectLike(object) {
      for (var prop in object) {
         var val = object[prop];
         if (Array.isArray(val))
            this[prop] = CloneArray(val);
         else if (typeof val == "function")
            continue;
         else if ((typeof val == "object") && (val != null)) {
            this[prop] = knownObjectsMap.get(val);
            if (this[prop] === undefined)
               this[prop] = "UNKNOWN_OBJECT (" + val.toString() + ")";
         }
         else
            this[prop] = val;
      }
   }

   function CloneArray(array) {
      var dup = [];
      for (var i = 0, len = array.length; i < len; i++) {
         var val = array[i]
         if (typeof val == "undefined")
            throw new Error("Ugg. Sparce arrays are not supported. Sorry!");
         else if (Array.isArray(val))
            dup[i] = "UNKNOWN_ARRAY";
         else if (typeof val == "function")
            dup[i] = "UNKNOWN_FUNCTION";
         else if ((typeof val == "object") && (val != null)) {
            dup[i] = knownObjectsMap.get(val);
            if (dup[i] === undefined)
               dup[i] = "UNKNOWN_OBJECT (" + val.toString() + ")";
         }
         else
            dup[i] = val;
      }
      return dup;
   }

   function generateRecordedEventHandlerWithCallback(callback) {
      return function(e) {
         if (recording) {
            // Setup the scope for any synchronous events
            eventsInScope.push(recordEvent(e));
            callback.call(this, e);
            eventsInScope.pop();
         }
      }
   }

   function recordedEventHandler(e) {
      if (recording)
         recordEvent(e);
   }

   function recordEvent(e) {
      var record = new EventRecord(e);
      var recordList = allRecords;
      // Adjust which sequential list to use depending on scope
      if (eventsInScope.length > 0) {
         recordList = eventsInScope[eventsInScope.length - 1].nestedEvents;
         if (recordList == null) // This top-of-stack event record hasn't had any nested events yet.
            recordList = eventsInScope[eventsInScope.length - 1].nestedEvents = [];
      }
      if (mergeTypesTruthMap[e.type] && (recordList.length > 0)) {
         var tail = recordList[recordList.length-1];
         // Same type and currentTarget?
         if ((tail.event.type == record.event.type) && (tail.event.currentTarget == record.event.currentTarget)) {
            tail.sequentialOccurrences++;
            return;
         }
      }
      recordList.push(record);
      return record;
   }

   Object.defineProperties(Node.prototype, {
      addRecordedEventListener: {
         enumerable: true, writable: true, configurable: true,
         value: function addRecordedEventListener(type, handler, capture) {
            if (handler == null)
               this.addEventListener(type, recordedEventHandler, capture);
            else {
               var subvertedHandler = generateRecordedEventHandlerWithCallback(handler);
               handlerMap.set(handler, subvertedHandler);
               this.addEventListener(type, subvertedHandler, capture);
            }
         }
      },
      removeRecordedEventListener: {
         enumerable: true, writable: true, configurable: true,
         value: function addRecordedEventListener(type, handler, capture) {
            var alternateHandlerUsed = handlerMap.get(handler);
            this.removeEventListenter(type, alternateHandlerUsed ? alternateHandlerUsed : recordedEventHandler, capture);
         }
      }
   });

})(window);