"use strict";

var allEvents = { events: [] };
const urlParams = new URLSearchParams(window.location.search);

const EVENTS = {
  wheel: "#0dcaf0",
  scroll: "#198754",
  scrollend: "#ffc107",
};

const SCROLL_EVENTS = ["scroll", "scrollend"];

// Event types for which listeners are registered. Defaults to all known
// events, but can be restricted through the "events" URL parameter as a
// comma-separated list (e.g. "events=wheel,scroll").
const registeredEvents = (() => {
  const param = urlParams.get("events");
  if (param === null) {
    return Object.keys(EVENTS);
  }
  return param.split(",").filter(type => type in EVENTS);
})();

function shouldRecordEvent(eventType) {
  if (!registeredEvents.includes(eventType)) return false;

  const showWheel = document.getElementById("showWheelEvents").checked;
  const showScroll = document.getElementById("showScrollEvents").checked;

  if (eventType === "wheel") return showWheel;
  if (SCROLL_EVENTS.includes(eventType)) return showScroll;

  return true;
}

function addEventToTable(eventData) {
  const tbody = document.getElementById("eventsTableBody");
  const row = tbody.insertRow(0);
  row.className = `event-${eventData.type}`;

  row.insertCell().textContent = allEvents.events.length;
  row.insertCell().textContent = eventData.type;
  row.insertCell().textContent = eventData.type === "wheel"
    ? `${eventData.pageX}, ${eventData.pageY}`
    : "-";
  row.insertCell().textContent = eventData.type === "wheel"
    ? `${eventData.deltaX}, ${eventData.deltaY}, ${eventData.deltaZ}`
    : "-";
  row.insertCell().textContent = eventData.deltaMode ?? "-";
  row.insertCell().textContent = eventData.target;
  row.insertCell().textContent = eventData.scrollLeft ?? "-";
  row.insertCell().textContent = eventData.scrollTop ?? "-";

  const modifiers = [];
  if (eventData.altKey) modifiers.push("Alt");
  if (eventData.ctrlKey) modifiers.push("Ctrl");
  if (eventData.metaKey) modifiers.push("Meta");
  if (eventData.shiftKey) modifiers.push("Shift");
  row.insertCell().textContent = modifiers.join("+") || "-";
}

function drawMarker(x, y, options = {}) {
  const { color = "#0dcaf0" } = options;

  const marker = document.createElement("div");
  marker.className = "marker";
  marker.style.left = `${x}px`;
  marker.style.top = `${y}px`;
  marker.style.backgroundColor = color;
  document.body.appendChild(marker);
}

function recordWheelEvent(eventOrData) {
  if (!shouldRecordEvent(eventOrData.type)) return;

  const isEvent = eventOrData instanceof Event;
  const target = isEvent
    ? eventOrData.target.id ||
      eventOrData.target.localName ||
      eventOrData.target.documentElement?.localName
    : eventOrData.target;

  const eventData = {
    type: eventOrData.type,
    button: eventOrData.button,
    buttons: eventOrData.buttons,
    pageX: eventOrData.pageX,
    pageY: eventOrData.pageY,
    deltaX: eventOrData.deltaX,
    deltaY: eventOrData.deltaY,
    deltaZ: eventOrData.deltaZ,
    deltaMode: eventOrData.deltaMode,
    target,
    altKey: eventOrData.altKey,
    ctrlKey: eventOrData.ctrlKey,
    metaKey: eventOrData.metaKey,
    shiftKey: eventOrData.shiftKey,
  };

  allEvents.events.push(eventData);
  addEventToTable(eventData);

  if (isEvent) {
    drawMarker(eventOrData.pageX, eventOrData.pageY, {
      color: EVENTS.wheel,
    });
  }
}

function recordScrollEvent(eventOrData) {
  if (!shouldRecordEvent(eventOrData.type)) return;

  const isEvent = eventOrData instanceof Event;
  const target = isEvent
    ? eventOrData.target.id ||
      eventOrData.target.localName ||
      eventOrData.target.documentElement?.localName
    : eventOrData.target;

  const eventData = {
    type: eventOrData.type,
    target,
    scrollLeft: isEvent
      ? eventOrData.target.scrollLeft
      : eventOrData.scrollLeft,
    scrollTop: isEvent
      ? eventOrData.target.scrollTop
      : eventOrData.scrollTop,
  };

  allEvents.events.push(eventData);
  addEventToTable(eventData);
}

function initializeFilters() {
  document.getElementById("showWheelEvents").checked =
    registeredEvents.includes("wheel");
  document.getElementById("showScrollEvents").checked =
    SCROLL_EVENTS.some(type => registeredEvents.includes(type));
}

function registerEventListeners(element) {
  if (registeredEvents.includes("wheel")) {
    element.addEventListener("wheel", recordWheelEvent);
  }
  if (registeredEvents.includes("scroll")) {
    element.addEventListener("scroll", recordScrollEvent);
  }
  if (registeredEvents.includes("scrollend")) {
    element.addEventListener("scrollend", recordScrollEvent);
  }
}

function initializeHandlers() {
  registerEventListeners(document.getElementById("not-scrollable"));
  registerEventListeners(document.getElementById("scrollable"));

  window.addEventListener("message", event => {
    if (event.data.action === "recordWheelEvent") {
      recordWheelEvent(event.data.data);
    } else if (event.data.action === "recordScrollEvent") {
      recordScrollEvent(event.data.data);
    }
  });
}

customElements.define("custom-scroll-element",
  class extends HTMLElement {
    constructor() {
      super();
      const mode = urlParams.get("shadow") || "closed";
      const shadowRoot = this.attachShadow({ mode });
      shadowRoot.innerHTML = `
        <style>
          div { margin: 0; padding: 0; }
          #shadow-scrollable {
            width: 100px; height: 100px; overflow: scroll;
          }
          #shadow-scrollable-content {
            width: 600px; height: 1000px; background-color: blue;
          }
        </style>
        <div id="shadow-scrollable">
          <div id="shadow-scrollable-content"></div>
        </div>`;

      const scrollable = shadowRoot.getElementById("shadow-scrollable");
      registerEventListeners(scrollable);
    }
  }
);

document.addEventListener("DOMContentLoaded", () => {
  allEvents.events.length = 0;

  const tbody = document.getElementById("eventsTableBody");
  if (tbody) {
    tbody.innerHTML = "";
  }

  const iframe = document.getElementById("iframe");
  const iframeOrigin = urlParams.get("iframe_origin") || window.location.origin;
  const iframeParams = new URLSearchParams();
  iframeParams.set("events", registeredEvents.join(","));
  iframe.src = `${iframeOrigin}/webdriver/tests/support/html/wheel/test_actions_inner_frame.html?${iframeParams}`;

  initializeHandlers();
  initializeFilters();
});
