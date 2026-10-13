"use strict";

const ALL_EVENTS = ["wheel", "scroll", "scrollend"];

// Event types for which listeners are registered. Mirrors the "events" URL
// parameter used by the parent document so both register the same events.
const registeredEvents = (() => {
  const param = new URLSearchParams(window.location.search).get("events");
  if (param === null) {
    return ALL_EVENTS;
  }
  return param.split(",").filter(type => ALL_EVENTS.includes(type));
})();

function relayWheelEvent(event) {
  window.parent.postMessage({
    "action": "recordWheelEvent",
    "data": {
      "type": event.type,
      "button": event.button,
      "buttons": event.buttons,
      "pageX": event.pageX,
      "pageY": event.pageY,
      "deltaX": event.deltaX,
      "deltaY": event.deltaY,
      "deltaZ": event.deltaZ,
      "deltaMode": event.deltaMode,
      "target": event.target.id ||
                event.target.localName ||
                event.target.documentElement?.localName,
      "altKey": event.altKey,
      "ctrlKey": event.ctrlKey,
      "metaKey": event.metaKey,
      "shiftKey": event.shiftKey,
    },
  }, "*");
}

function relayScrollEvent(event) {
  window.parent.postMessage({
    "action": "recordScrollEvent",
    "data": {
      "type": event.type,
      "target": event.target.id ||
                event.target.localName ||
                event.target.documentElement?.localName,
      "scrollLeft": event.target.scrollLeft,
      "scrollTop": event.target.scrollTop,
    },
  }, "*");
}

function registerEventListeners(element) {
  if (registeredEvents.includes("wheel")) {
    element.addEventListener("wheel", relayWheelEvent);
  }
  if (registeredEvents.includes("scroll")) {
    element.addEventListener("scroll", relayScrollEvent);
  }
  if (registeredEvents.includes("scrollend")) {
    element.addEventListener("scrollend", relayScrollEvent);
  }
}

document.addEventListener("DOMContentLoaded", () => {
  registerEventListeners(document.getElementById("inner-not-scrollable"));
  registerEventListeners(document.getElementById("inner-scrollable"));
});
