# mypy: allow-untyped-defs

import time
import uuid

from threading import Timer

from ..data.device import Device
from ..testing.event_dispatcher import DEVICES, DEVICE_ADDED_EVENT, DEVICE_REMOVED_EVENT
from ..utils.user_agent_parser import parse_user_agent
from ..utils.serializer import serialize_device
from ..data.exceptions.not_found_exception import NotFoundException


DEVICE_TIMEOUT = 60000  # 60sec
RECONNECT_TIME = 5000   # 5sec

class DevicesManager:
    def initialize(self, event_dispatcher):
        self.devices = {}
        self._event_dispatcher = event_dispatcher
        self._timer = None

    def create_device(self, user_agent):
        browser = parse_user_agent(user_agent)
        name = "{} {}".format(browser["name"], browser["version"])
        token = str(uuid.uuid1())
        last_active = int(time.time() * 1000)

        device = Device(token, user_agent, name, last_active)

        self._event_dispatcher.dispatch_event(
            DEVICES,
            DEVICE_ADDED_EVENT,
            serialize_device(device))
        self.add_to_cache(device)

        self._set_timer(DEVICE_TIMEOUT)

        return device

    def read_device(self, token):
        if token not in self.devices:
            raise NotFoundException(f"Could not find device '{token}'")
        return self.devices[token]

    def read_devices(self):
        devices = []
        for key in self.devices:
            devices.append(self.devices[key])

        return devices

    def update_device(self, device):
        if device.token not in self.devices:
            return
        self.devices[device.token] = device

    def delete_device(self, token):
        if token not in self.devices:
            return
        device = self.devices[token]
        del self.devices[token]
        self._event_dispatcher.dispatch_event(
            DEVICES,
            DEVICE_REMOVED_EVENT,
            serialize_device(device))

    def refresh_device(self, token):
        if token not in self.devices:
            return
        device = self.devices[token]
        device.last_active = int(time.time() * 1000)
        self.update_device(device)

    def post_event(self, handle, event_type, data):
        if event_type is None:
            return
        self._event_dispatcher.dispatch_event(handle, event_type, data)

    def post_global_event(self, event_type, data):
        self.post_event(DEVICES, event_type, data)

    def _set_timer(self, timeout):
        if self._timer is not None:
            return

        def handle_timeout(self):
            self._timer = None
            now = int(time.time() * 1000)
            timed_out_devices = []
            for token in self.devices:
                device = self.devices[token]
                if now - device.last_active < DEVICE_TIMEOUT:
                    continue
                timed_out_devices.append(token)

            for token in timed_out_devices:
                self.delete_device(token)

            oldest_active_time = None
            for token in self.devices:
                device = self.devices[token]
                if oldest_active_time is None:
                    oldest_active_time = device.last_active
                else:
                    if oldest_active_time > device.last_active:
                        oldest_active_time = device.last_active
            if oldest_active_time is not None:
                self._set_timer(now - oldest_active_time)

        self._timer = Timer(timeout / 1000.0, handle_timeout, [self])
        self._timer.start()

    def add_to_cache(self, device):
        if device.token in self.devices:
            return

        self.devices[device.token] = device
