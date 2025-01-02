# mypy: allow-untyped-defs

import json
import threading

from .api_handler import ApiHandler
from ...data.http_polling_event_listener import HttpPollingEventListener
from ...testing.event_dispatcher import DEVICES
from ...utils.serializer import serialize_device
from ...testing.devices_manager import DEVICE_TIMEOUT, RECONNECT_TIME
from ...data.exceptions.not_found_exception import NotFoundException


class DevicesApiHandler(ApiHandler):
    def __init__(self, devices_manager, event_dispatcher, web_root):
        super().__init__(web_root)
        self._devices_manager = devices_manager
        self._event_dispatcher = event_dispatcher

    def create_device(self, request, response):
        try:
            user_agent = request.headers[b"user-agent"].decode("utf-8")

            device = self._devices_manager.create_device(user_agent)

            self.send_json({"token": device.token}, response)
        except Exception:
            self.handle_exception("Failed to create device")
            response.status = 500

    def read_device(self, request, response):
        try:
            uri_parts = self.parse_uri(request)
            token = uri_parts[2]

            device = self._devices_manager.read_device(token)

            device_object = serialize_device(device)

            self.send_json(device_object, response)
        except NotFoundException:
            self.handle_exception("Failed to read device")
            response.status = 404
        except Exception:
            self.handle_exception("Failed to read device")
            response.status = 500

    def read_devices(self, request, response):
        try:
            devices = self._devices_manager.read_devices()

            device_objects = []
            for device in devices:
                device_object = serialize_device(device)
                device_objects.append(device_object)

            self.send_json(device_objects, response)
        except Exception:
            self.handle_exception("Failed to read devices")
            response.status = 500

    def register_event_listener(self, request, response):
        try:
            uri_parts = self.parse_uri(request)
            token = uri_parts[2]
            query = self.parse_query_parameters(request)

            if "device_token" in query:
                self._devices_manager.refresh_device(query["device_token"])

            event = threading.Event()
            timer = threading.Timer(
                (DEVICE_TIMEOUT - RECONNECT_TIME) / 1000,
                event.set,
                [])
            timer.start()
            http_polling_event_listener = HttpPollingEventListener(token, event)
            event_listener_token = self._event_dispatcher.add_event_listener(http_polling_event_listener)

            event.wait()

            message = http_polling_event_listener.message
            if message is not None:
                self.send_json(data=message, response=response)
            self._event_dispatcher.remove_event_listener(event_listener_token)
        except Exception:
            self.handle_exception("Failed to register event listener")
            response.status = 500

    def register_global_event_listener(self, request, response):
        try:
            query = self.parse_query_parameters(request)

            if "device_token" in query:
                self._devices_manager.refresh_device(query["device_token"])

            event = threading.Event()
            timer = threading.Timer(
                (DEVICE_TIMEOUT - RECONNECT_TIME) / 1000,
                event.set,
                [])
            timer.start()
            http_polling_event_listener = HttpPollingEventListener(DEVICES, event)
            event_listener_token = self._event_dispatcher.add_event_listener(http_polling_event_listener)

            event.wait()

            message = http_polling_event_listener.message
            if message is not None:
                self.send_json(data=message, response=response)
            self._event_dispatcher.remove_event_listener(event_listener_token)
        except Exception:
            self.handle_exception("Failed to register global event listener")
            response.status = 500

    def post_global_event(self, request, response):
        try:
            event = {}
            body = request.body.decode("utf-8")
            if body != "":
                event = json.loads(body)

            query = self.parse_query_parameters(request)
            if "device_token" in query:
                self._devices_manager.refresh_device(query["device_token"])

            event_type = None
            if "type" in event:
                event_type = event["type"]
            data = None
            if "data" in event:
                data = event["data"]
            self._devices_manager.post_global_event(event_type, data)

        except Exception:
            self.handle_exception("Failed to post global event")
            response.status = 500

    def post_event(self, request, response):
        try:
            uri_parts = self.parse_uri(request)
            token = uri_parts[2]

            event = {}
            body = request.body.decode("utf-8")
            if body != "":
                event = json.loads(body)

            query = self.parse_query_parameters(request)
            if "device_token" in query:
                self._devices_manager.refresh_device(query["device_token"])

            event_type = None
            if "type" in event:
                event_type = event["type"]
            data = None
            if "data" in event:
                data = event["data"]
            self._devices_manager.post_event(token, event_type, data)

        except Exception:
            self.handle_exception("Failed to post event")
            response.status = 500

    def handle_request(self, request, response):
        method = request.method
        uri_parts = self.parse_uri(request)

        # /api/devices
        if len(uri_parts) == 2:
            if method == "POST":
                self.create_device(request, response)
                return
            if method == "GET":
                self.read_devices(request, response)
                return

        # /api/devices/<function>
        if len(uri_parts) == 3:
            function = uri_parts[2]
            if method == "GET":
                if function == "events":
                    self.register_global_event_listener(request, response)
                    return
                self.read_device(request, response)
                return
            if method == "POST":
                if function == "events":
                    self.post_global_event(request, response)
                    return

        # /api/devices/<token>/<function>
        if len(uri_parts) == 4:
            function = uri_parts[3]
            if method == "GET":
                if function == "events":
                    self.register_event_listener(request, response)
                    return
            if method == "POST":
                if function == "events":
                    self.post_event(request, response)
                    return
