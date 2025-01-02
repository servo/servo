# mypy: allow-untyped-defs

import uuid
import time
from threading import Timer


STATUS_EVENT = "status"
RESUME_EVENT = "resume"
TEST_COMPLETED_EVENT = "test_completed"

DEVICES = "devices"
DEVICE_ADDED_EVENT = "device_added"
DEVICE_REMOVED_EVENT = "device_removed"

class EventDispatcher:
    def __init__(self, event_cache_duration):
        self._listeners = {}
        self._events = {}
        self._current_events = {}
        self._cache_duration = event_cache_duration
        self._cache_timeout = None

    def add_event_listener(self, listener, last_event_number=None):
        token = listener.dispatcher_token

        if last_event_number is not None \
            and token in self._current_events \
                and self._current_events[token] > last_event_number:
            diff_events = self._get_diff_events(token, last_event_number)
            if len(diff_events) > 0:
                listener.send_message(diff_events)
                return

        if token not in self._listeners:
            self._listeners[token] = []
        self._listeners[token].append(listener)
        listener.token = str(uuid.uuid1())
        return listener.token

    def remove_event_listener(self, listener_token):
        if listener_token is None:
            return

        for dispatcher_token in self._listeners:
            for listener in self._listeners[dispatcher_token]:
                if listener.token == listener_token:
                    self._listeners[dispatcher_token].remove(listener)
                    if len(self._listeners[dispatcher_token]) == 0:
                        del self._listeners[dispatcher_token]
                    return

    def dispatch_event(self, dispatcher_token, event_type, data=None):
        if dispatcher_token not in self._current_events:
            self._current_events[dispatcher_token] = -1

        if dispatcher_token not in self._events:
            self._events[dispatcher_token] = []

        self._add_to_cache(dispatcher_token, event_type, data)
        self._set_cache_timer()

        if dispatcher_token not in self._listeners:
            return

        event = {
            "type": event_type,
            "data": data,
            "number": self._current_events[dispatcher_token]
        }

        for listener in self._listeners[dispatcher_token]:
            listener.send_message([event])

    def _get_diff_events(self, dispatcher_token, last_event_number):
        token = dispatcher_token
        diff_events = []
        if token not in self._events:
            return diff_events
        for event in self._events[token]:
            if event["number"] <= last_event_number:
                continue
            diff_events.append({
                "type": event["type"],
                "data": event["data"],
                "number": event["number"]
            })
        return diff_events

    def _set_cache_timer(self):
        if self._cache_timeout is not None:
            return

        events = self._read_cached_events()
        if len(events) == 0:
            return

        next_event = events[0]
        for event in events:
            if next_event["expiration_date"] > event["expiration_date"]:
                next_event = event

        timeout = next_event["expiration_date"] / 1000.0 - time.time()
        if timeout < 0:
            timeout = 0

        def handle_timeout(self):
            self._delete_expired_events()
            self._cache_timeout = None
            self._set_cache_timer()

        self._cache_timeout = Timer(timeout, handle_timeout, [self])
        self._cache_timeout.start()

    def _delete_expired_events(self):
        events = self._read_cached_events()
        now = int(time.time() * 1000)

        for event in events:
            if event["expiration_date"] < now:
                self._remove_from_cache(event)

    def _add_to_cache(self, dispatcher_token, event_type, data):
        self._current_events[dispatcher_token] += 1
        current_event_number = self._current_events[dispatcher_token]
        event = {
            "type": event_type,
            "data": data,
            "number": current_event_number,
            "expiration_date": int(time.time() * 1000) + self._cache_duration
        }
        self._events[dispatcher_token].append(event)

    def _remove_from_cache(self, event):
        for dispatcher_token in self._events:
            for cached_event in self._events[dispatcher_token]:
                if cached_event is not event:
                    continue
                self._events[dispatcher_token].remove(cached_event)
                if len(self._events[dispatcher_token]) == 0:
                    del self._events[dispatcher_token]
                return

    def _read_cached_events(self):
        events = []
        for dispatcher_token in self._events:
            events = events + self._events[dispatcher_token]
        return events
