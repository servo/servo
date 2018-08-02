# -*- coding: utf-8 -*-
"""
test_events.py
~~~~~~~~~~~~~~

Specific tests for any function that is logically self-contained as part of
events.py.
"""
import inspect
import sys

from hypothesis import given
from hypothesis.strategies import (
    integers, lists, tuples
)
import pytest

import h2.errors
import h2.events
import h2.settings


# We define a fairly complex Hypothesis strategy here. We want to build a list
# of two tuples of (Setting, value). For Setting we want to make sure we can
# handle settings that the rest of hyper knows nothing about, so we want to
# use integers from 0 to (2**16-1). For values, they're from 0 to (2**32-1).
# Define that strategy here for clarity.
SETTINGS_STRATEGY = lists(
    tuples(
        integers(min_value=0, max_value=2**16-1),
        integers(min_value=0, max_value=2**32-1),
    )
)


class TestRemoteSettingsChanged(object):
    """
    Validate the function of the RemoteSettingsChanged event.
    """
    @given(SETTINGS_STRATEGY)
    def test_building_settings_from_scratch(self, settings_list):
        """
        Missing old settings are defaulted to None.
        """
        settings_dict = dict(settings_list)
        e = h2.events.RemoteSettingsChanged.from_settings(
            old_settings={},
            new_settings=settings_dict,
        )

        for setting, new_value in settings_dict.items():
            assert e.changed_settings[setting].setting == setting
            assert e.changed_settings[setting].original_value is None
            assert e.changed_settings[setting].new_value == new_value

    @given(SETTINGS_STRATEGY, SETTINGS_STRATEGY)
    def test_only_reports_changed_settings(self,
                                           old_settings_list,
                                           new_settings_list):
        """
        Settings that were not changed are not reported.
        """
        old_settings_dict = dict(old_settings_list)
        new_settings_dict = dict(new_settings_list)
        e = h2.events.RemoteSettingsChanged.from_settings(
            old_settings=old_settings_dict,
            new_settings=new_settings_dict,
        )

        assert len(e.changed_settings) == len(new_settings_dict)
        assert (
            sorted(list(e.changed_settings.keys())) ==
            sorted(list(new_settings_dict.keys()))
        )

    @given(SETTINGS_STRATEGY, SETTINGS_STRATEGY)
    def test_correctly_reports_changed_settings(self,
                                                old_settings_list,
                                                new_settings_list):
        """
        Settings that are changed are correctly reported.
        """
        old_settings_dict = dict(old_settings_list)
        new_settings_dict = dict(new_settings_list)
        e = h2.events.RemoteSettingsChanged.from_settings(
            old_settings=old_settings_dict,
            new_settings=new_settings_dict,
        )

        for setting, new_value in new_settings_dict.items():
            original_value = old_settings_dict.get(setting)
            assert e.changed_settings[setting].setting == setting
            assert e.changed_settings[setting].original_value == original_value
            assert e.changed_settings[setting].new_value == new_value


class TestEventReprs(object):
    """
    Events have useful representations.
    """
    example_request_headers = [
        (':authority', 'example.com'),
        (':path', '/'),
        (':scheme', 'https'),
        (':method', 'GET'),
    ]
    example_informational_headers = [
        (':status', '100'),
        ('server', 'fake-serv/0.1.0')
    ]
    example_response_headers = [
        (':status', '200'),
        ('server', 'fake-serv/0.1.0')
    ]

    def test_requestreceived_repr(self):
        """
        RequestReceived has a useful debug representation.
        """
        e = h2.events.RequestReceived()
        e.stream_id = 5
        e.headers = self.example_request_headers

        assert repr(e) == (
            "<RequestReceived stream_id:5, headers:["
            "(':authority', 'example.com'), "
            "(':path', '/'), "
            "(':scheme', 'https'), "
            "(':method', 'GET')]>"
        )

    def test_responsereceived_repr(self):
        """
        ResponseReceived has a useful debug representation.
        """
        e = h2.events.ResponseReceived()
        e.stream_id = 500
        e.headers = self.example_response_headers

        assert repr(e) == (
            "<ResponseReceived stream_id:500, headers:["
            "(':status', '200'), "
            "('server', 'fake-serv/0.1.0')]>"
        )

    def test_trailersreceived_repr(self):
        """
        TrailersReceived has a useful debug representation.
        """
        e = h2.events.TrailersReceived()
        e.stream_id = 62
        e.headers = self.example_response_headers

        assert repr(e) == (
            "<TrailersReceived stream_id:62, headers:["
            "(':status', '200'), "
            "('server', 'fake-serv/0.1.0')]>"
        )

    def test_informationalresponsereceived_repr(self):
        """
        InformationalResponseReceived has a useful debug representation.
        """
        e = h2.events.InformationalResponseReceived()
        e.stream_id = 62
        e.headers = self.example_informational_headers

        assert repr(e) == (
            "<InformationalResponseReceived stream_id:62, headers:["
            "(':status', '100'), "
            "('server', 'fake-serv/0.1.0')]>"
        )

    def test_datareceived_repr(self):
        """
        DataReceived has a useful debug representation.
        """
        e = h2.events.DataReceived()
        e.stream_id = 888
        e.data = b"abcdefghijklmnopqrstuvwxyz"
        e.flow_controlled_length = 88

        assert repr(e) == (
            "<DataReceived stream_id:888, flow_controlled_length:88, "
            "data:6162636465666768696a6b6c6d6e6f7071727374>"
        )

    def test_windowupdated_repr(self):
        """
        WindowUpdated has a useful debug representation.
        """
        e = h2.events.WindowUpdated()
        e.stream_id = 0
        e.delta = 2**16

        assert repr(e) == "<WindowUpdated stream_id:0, delta:65536>"

    def test_remotesettingschanged_repr(self):
        """
        RemoteSettingsChanged has a useful debug representation.
        """
        e = h2.events.RemoteSettingsChanged()
        e.changed_settings = {
            h2.settings.SettingCodes.INITIAL_WINDOW_SIZE:
                h2.settings.ChangedSetting(
                    h2.settings.SettingCodes.INITIAL_WINDOW_SIZE, 2**16, 2**15
                ),
        }

        assert repr(e) == (
            "<RemoteSettingsChanged changed_settings:{ChangedSetting("
            "setting=SettingCodes.INITIAL_WINDOW_SIZE, original_value=65536, "
            "new_value=32768)}>"
        )

    def test_pingacknowledged_repr(self):
        """
        PingAcknowledged has a useful debug representation.
        """
        e = h2.events.PingAcknowledged()
        e.ping_data = b'abcdefgh'

        assert repr(e) == "<PingAcknowledged ping_data:6162636465666768>"

    def test_streamended_repr(self):
        """
        StreamEnded has a useful debug representation.
        """
        e = h2.events.StreamEnded()
        e.stream_id = 99

        assert repr(e) == "<StreamEnded stream_id:99>"

    def test_streamreset_repr(self):
        """
        StreamEnded has a useful debug representation.
        """
        e = h2.events.StreamReset()
        e.stream_id = 919
        e.error_code = h2.errors.ErrorCodes.ENHANCE_YOUR_CALM
        e.remote_reset = False

        assert repr(e) == (
            "<StreamReset stream_id:919, "
            "error_code:ErrorCodes.ENHANCE_YOUR_CALM, remote_reset:False>"
        )

    def test_pushedstreamreceived_repr(self):
        """
        PushedStreamReceived has a useful debug representation.
        """
        e = h2.events.PushedStreamReceived()
        e.pushed_stream_id = 50
        e.parent_stream_id = 11
        e.headers = self.example_request_headers

        assert repr(e) == (
            "<PushedStreamReceived pushed_stream_id:50, parent_stream_id:11, "
            "headers:["
            "(':authority', 'example.com'), "
            "(':path', '/'), "
            "(':scheme', 'https'), "
            "(':method', 'GET')]>"
        )

    def test_settingsacknowledged_repr(self):
        """
        SettingsAcknowledged has a useful debug representation.
        """
        e = h2.events.SettingsAcknowledged()
        e.changed_settings = {
            h2.settings.SettingCodes.INITIAL_WINDOW_SIZE:
                h2.settings.ChangedSetting(
                    h2.settings.SettingCodes.INITIAL_WINDOW_SIZE, 2**16, 2**15
                ),
        }

        assert repr(e) == (
            "<SettingsAcknowledged changed_settings:{ChangedSetting("
            "setting=SettingCodes.INITIAL_WINDOW_SIZE, original_value=65536, "
            "new_value=32768)}>"
        )

    def test_priorityupdated_repr(self):
        """
        PriorityUpdated has a useful debug representation.
        """
        e = h2.events.PriorityUpdated()
        e.stream_id = 87
        e.weight = 32
        e.depends_on = 8
        e.exclusive = True

        assert repr(e) == (
            "<PriorityUpdated stream_id:87, weight:32, depends_on:8, "
            "exclusive:True>"
        )

    @pytest.mark.parametrize("additional_data,data_repr", [
        (None, "None"),
        (b'some data', "736f6d652064617461")
    ])
    def test_connectionterminated_repr(self, additional_data, data_repr):
        """
        ConnectionTerminated has a useful debug representation.
        """
        e = h2.events.ConnectionTerminated()
        e.error_code = h2.errors.ErrorCodes.INADEQUATE_SECURITY
        e.last_stream_id = 33
        e.additional_data = additional_data

        assert repr(e) == (
            "<ConnectionTerminated error_code:ErrorCodes.INADEQUATE_SECURITY, "
            "last_stream_id:33, additional_data:%s>" % data_repr
        )

    def test_alternativeserviceavailable_repr(self):
        """
        AlternativeServiceAvailable has a useful debug representation.
        """
        e = h2.events.AlternativeServiceAvailable()
        e.origin = b"example.com"
        e.field_value = b'h2=":8000"; ma=60'

        assert repr(e) == (
            '<AlternativeServiceAvailable origin:example.com, '
            'field_value:h2=":8000"; ma=60>'
        )

    def test_unknownframereceived_repr(self):
        """
        UnknownFrameReceived has a useful debug representation.
        """
        e = h2.events.UnknownFrameReceived()
        assert repr(e) == '<UnknownFrameReceived>'


def all_events():
    """
    Generates all the classes (i.e., events) defined in h2.events.
    """
    for _, obj in inspect.getmembers(sys.modules['h2.events']):

        # We are only interested in objects that are defined in h2.events;
        # objects that are imported from other modules are not of interest.
        if hasattr(obj, '__module__') and (obj.__module__ != 'h2.events'):
            continue

        if inspect.isclass(obj):
            yield obj


@pytest.mark.parametrize('event', all_events())
def test_all_events_subclass_from_event(event):
    """
    Every event defined in h2.events subclasses from h2.events.Event.
    """
    assert (event is h2.events.Event) or issubclass(event, h2.events.Event)
