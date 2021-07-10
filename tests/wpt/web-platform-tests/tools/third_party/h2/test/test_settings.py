# -*- coding: utf-8 -*-
"""
test_settings
~~~~~~~~~~~~~

Test the Settings object.
"""
import pytest

import h2.errors
import h2.exceptions
import h2.settings

from hypothesis import given, assume
from hypothesis.strategies import (
    integers, booleans, fixed_dictionaries, builds
)


class TestSettings(object):
    """
    Test the Settings object behaves as expected.
    """
    def test_settings_defaults_client(self):
        """
        The Settings object begins with the appropriate defaults for clients.
        """
        s = h2.settings.Settings(client=True)

        assert s[h2.settings.SettingCodes.HEADER_TABLE_SIZE] == 4096
        assert s[h2.settings.SettingCodes.ENABLE_PUSH] == 1
        assert s[h2.settings.SettingCodes.INITIAL_WINDOW_SIZE] == 65535
        assert s[h2.settings.SettingCodes.MAX_FRAME_SIZE] == 16384
        assert s[h2.settings.SettingCodes.ENABLE_CONNECT_PROTOCOL] == 0

    def test_settings_defaults_server(self):
        """
        The Settings object begins with the appropriate defaults for servers.
        """
        s = h2.settings.Settings(client=False)

        assert s[h2.settings.SettingCodes.HEADER_TABLE_SIZE] == 4096
        assert s[h2.settings.SettingCodes.ENABLE_PUSH] == 0
        assert s[h2.settings.SettingCodes.INITIAL_WINDOW_SIZE] == 65535
        assert s[h2.settings.SettingCodes.MAX_FRAME_SIZE] == 16384
        assert s[h2.settings.SettingCodes.ENABLE_CONNECT_PROTOCOL] == 0

    @pytest.mark.parametrize('client', [True, False])
    def test_can_set_initial_values(self, client):
        """
        The Settings object can be provided initial values that override the
        defaults.
        """
        overrides = {
            h2.settings.SettingCodes.HEADER_TABLE_SIZE: 8080,
            h2.settings.SettingCodes.MAX_FRAME_SIZE: 16388,
            h2.settings.SettingCodes.MAX_CONCURRENT_STREAMS: 100,
            h2.settings.SettingCodes.MAX_HEADER_LIST_SIZE: 2**16,
            h2.settings.SettingCodes.ENABLE_CONNECT_PROTOCOL: 1,
        }
        s = h2.settings.Settings(client=client, initial_values=overrides)

        assert s[h2.settings.SettingCodes.HEADER_TABLE_SIZE] == 8080
        assert s[h2.settings.SettingCodes.ENABLE_PUSH] == bool(client)
        assert s[h2.settings.SettingCodes.INITIAL_WINDOW_SIZE] == 65535
        assert s[h2.settings.SettingCodes.MAX_FRAME_SIZE] == 16388
        assert s[h2.settings.SettingCodes.MAX_CONCURRENT_STREAMS] == 100
        assert s[h2.settings.SettingCodes.MAX_HEADER_LIST_SIZE] == 2**16
        assert s[h2.settings.SettingCodes.ENABLE_CONNECT_PROTOCOL] == 1

    @pytest.mark.parametrize(
        'setting,value',
        [
            (h2.settings.SettingCodes.ENABLE_PUSH, 2),
            (h2.settings.SettingCodes.ENABLE_PUSH, -1),
            (h2.settings.SettingCodes.INITIAL_WINDOW_SIZE, -1),
            (h2.settings.SettingCodes.INITIAL_WINDOW_SIZE, 2**34),
            (h2.settings.SettingCodes.MAX_FRAME_SIZE, 1),
            (h2.settings.SettingCodes.MAX_FRAME_SIZE, 2**30),
            (h2.settings.SettingCodes.MAX_HEADER_LIST_SIZE, -1),
            (h2.settings.SettingCodes.ENABLE_CONNECT_PROTOCOL, -1),
        ]
    )
    def test_cannot_set_invalid_initial_values(self, setting, value):
        """
        The Settings object can be provided initial values that override the
        defaults.
        """
        overrides = {setting: value}

        with pytest.raises(h2.exceptions.InvalidSettingsValueError):
            h2.settings.Settings(initial_values=overrides)

    def test_applying_value_doesnt_take_effect_immediately(self):
        """
        When a value is applied to the settings object, it doesn't immediately
        take effect.
        """
        s = h2.settings.Settings(client=True)
        s[h2.settings.SettingCodes.HEADER_TABLE_SIZE] == 8000

        assert s[h2.settings.SettingCodes.HEADER_TABLE_SIZE] == 4096

    def test_acknowledging_values(self):
        """
        When we acknowledge settings, the values change.
        """
        s = h2.settings.Settings(client=True)
        old_settings = dict(s)

        new_settings = {
            h2.settings.SettingCodes.HEADER_TABLE_SIZE: 4000,
            h2.settings.SettingCodes.ENABLE_PUSH: 0,
            h2.settings.SettingCodes.INITIAL_WINDOW_SIZE: 60,
            h2.settings.SettingCodes.MAX_FRAME_SIZE: 16385,
            h2.settings.SettingCodes.ENABLE_CONNECT_PROTOCOL: 1,
        }
        s.update(new_settings)

        assert dict(s) == old_settings
        s.acknowledge()
        assert dict(s) == new_settings

    def test_acknowledging_returns_the_changed_settings(self):
        """
        Acknowledging settings returns the changes.
        """
        s = h2.settings.Settings(client=True)
        s[h2.settings.SettingCodes.HEADER_TABLE_SIZE] = 8000
        s[h2.settings.SettingCodes.ENABLE_PUSH] = 0

        changes = s.acknowledge()
        assert len(changes) == 2

        table_size_change = (
            changes[h2.settings.SettingCodes.HEADER_TABLE_SIZE]
        )
        push_change = changes[h2.settings.SettingCodes.ENABLE_PUSH]

        assert table_size_change.setting == (
            h2.settings.SettingCodes.HEADER_TABLE_SIZE
        )
        assert table_size_change.original_value == 4096
        assert table_size_change.new_value == 8000

        assert push_change.setting == h2.settings.SettingCodes.ENABLE_PUSH
        assert push_change.original_value == 1
        assert push_change.new_value == 0

    def test_acknowledging_only_returns_changed_settings(self):
        """
        Acknowledging settings does not return unchanged settings.
        """
        s = h2.settings.Settings(client=True)
        s[h2.settings.SettingCodes.INITIAL_WINDOW_SIZE] = 70

        changes = s.acknowledge()
        assert len(changes) == 1
        assert list(changes.keys()) == [
            h2.settings.SettingCodes.INITIAL_WINDOW_SIZE
        ]

    def test_deleting_values_deletes_all_of_them(self):
        """
        When we delete a key we lose all state about it.
        """
        s = h2.settings.Settings(client=True)
        s[h2.settings.SettingCodes.HEADER_TABLE_SIZE] == 8000

        del s[h2.settings.SettingCodes.HEADER_TABLE_SIZE]

        with pytest.raises(KeyError):
            s[h2.settings.SettingCodes.HEADER_TABLE_SIZE]

    def test_length_correctly_reported(self):
        """
        Length is related only to the number of keys.
        """
        s = h2.settings.Settings(client=True)
        assert len(s) == 5

        s[h2.settings.SettingCodes.HEADER_TABLE_SIZE] == 8000
        assert len(s) == 5

        s.acknowledge()
        assert len(s) == 5

        del s[h2.settings.SettingCodes.HEADER_TABLE_SIZE]
        assert len(s) == 4

    def test_new_values_work(self):
        """
        New values initially don't appear
        """
        s = h2.settings.Settings(client=True)
        s[80] = 81

        with pytest.raises(KeyError):
            s[80]

    def test_new_values_follow_basic_acknowledgement_rules(self):
        """
        A new value properly appears when acknowledged.
        """
        s = h2.settings.Settings(client=True)
        s[80] = 81
        changed_settings = s.acknowledge()

        assert s[80] == 81
        assert len(changed_settings) == 1

        changed = changed_settings[80]
        assert changed.setting == 80
        assert changed.original_value is None
        assert changed.new_value == 81

    def test_single_values_arent_affected_by_acknowledgement(self):
        """
        When acknowledged, unchanged settings remain unchanged.
        """
        s = h2.settings.Settings(client=True)
        assert s[h2.settings.SettingCodes.HEADER_TABLE_SIZE] == 4096

        s.acknowledge()
        assert s[h2.settings.SettingCodes.HEADER_TABLE_SIZE] == 4096

    def test_settings_getters(self):
        """
        Getters exist for well-known settings.
        """
        s = h2.settings.Settings(client=True)

        assert s.header_table_size == (
            s[h2.settings.SettingCodes.HEADER_TABLE_SIZE]
        )
        assert s.enable_push == s[h2.settings.SettingCodes.ENABLE_PUSH]
        assert s.initial_window_size == (
            s[h2.settings.SettingCodes.INITIAL_WINDOW_SIZE]
        )
        assert s.max_frame_size == s[h2.settings.SettingCodes.MAX_FRAME_SIZE]
        assert s.max_concurrent_streams == 2**32 + 1  # A sensible default.
        assert s.max_header_list_size is None
        assert s.enable_connect_protocol == s[
            h2.settings.SettingCodes.ENABLE_CONNECT_PROTOCOL
        ]

    def test_settings_setters(self):
        """
        Setters exist for well-known settings.
        """
        s = h2.settings.Settings(client=True)

        s.header_table_size = 0
        s.enable_push = 1
        s.initial_window_size = 2
        s.max_frame_size = 16385
        s.max_concurrent_streams = 4
        s.max_header_list_size = 2**16
        s.enable_connect_protocol = 1

        s.acknowledge()
        assert s[h2.settings.SettingCodes.HEADER_TABLE_SIZE] == 0
        assert s[h2.settings.SettingCodes.ENABLE_PUSH] == 1
        assert s[h2.settings.SettingCodes.INITIAL_WINDOW_SIZE] == 2
        assert s[h2.settings.SettingCodes.MAX_FRAME_SIZE] == 16385
        assert s[h2.settings.SettingCodes.MAX_CONCURRENT_STREAMS] == 4
        assert s[h2.settings.SettingCodes.MAX_HEADER_LIST_SIZE] == 2**16
        assert s[h2.settings.SettingCodes.ENABLE_CONNECT_PROTOCOL] == 1

    @given(integers())
    def test_cannot_set_invalid_values_for_enable_push(self, val):
        """
        SETTINGS_ENABLE_PUSH only allows two values: 0, 1.
        """
        assume(val not in (0, 1))
        s = h2.settings.Settings()

        with pytest.raises(h2.exceptions.InvalidSettingsValueError) as e:
            s.enable_push = val

        s.acknowledge()
        assert e.value.error_code == h2.errors.ErrorCodes.PROTOCOL_ERROR
        assert s.enable_push == 1

        with pytest.raises(h2.exceptions.InvalidSettingsValueError) as e:
            s[h2.settings.SettingCodes.ENABLE_PUSH] = val

        s.acknowledge()
        assert e.value.error_code == h2.errors.ErrorCodes.PROTOCOL_ERROR
        assert s[h2.settings.SettingCodes.ENABLE_PUSH] == 1

    @given(integers())
    def test_cannot_set_invalid_vals_for_initial_window_size(self, val):
        """
        SETTINGS_INITIAL_WINDOW_SIZE only allows values between 0 and 2**32 - 1
        inclusive.
        """
        s = h2.settings.Settings()

        if 0 <= val <= 2**31 - 1:
            s.initial_window_size = val
            s.acknowledge()
            assert s.initial_window_size == val
        else:
            with pytest.raises(h2.exceptions.InvalidSettingsValueError) as e:
                s.initial_window_size = val

            s.acknowledge()
            assert (
                e.value.error_code == h2.errors.ErrorCodes.FLOW_CONTROL_ERROR
            )
            assert s.initial_window_size == 65535

            with pytest.raises(h2.exceptions.InvalidSettingsValueError) as e:
                s[h2.settings.SettingCodes.INITIAL_WINDOW_SIZE] = val

            s.acknowledge()
            assert (
                e.value.error_code == h2.errors.ErrorCodes.FLOW_CONTROL_ERROR
            )
            assert s[h2.settings.SettingCodes.INITIAL_WINDOW_SIZE] == 65535

    @given(integers())
    def test_cannot_set_invalid_values_for_max_frame_size(self, val):
        """
        SETTINGS_MAX_FRAME_SIZE only allows values between 2**14 and 2**24 - 1.
        """
        s = h2.settings.Settings()

        if 2**14 <= val <= 2**24 - 1:
            s.max_frame_size = val
            s.acknowledge()
            assert s.max_frame_size == val
        else:
            with pytest.raises(h2.exceptions.InvalidSettingsValueError) as e:
                s.max_frame_size = val

            s.acknowledge()
            assert e.value.error_code == h2.errors.ErrorCodes.PROTOCOL_ERROR
            assert s.max_frame_size == 16384

            with pytest.raises(h2.exceptions.InvalidSettingsValueError) as e:
                s[h2.settings.SettingCodes.MAX_FRAME_SIZE] = val

            s.acknowledge()
            assert e.value.error_code == h2.errors.ErrorCodes.PROTOCOL_ERROR
            assert s[h2.settings.SettingCodes.MAX_FRAME_SIZE] == 16384

    @given(integers())
    def test_cannot_set_invalid_values_for_max_header_list_size(self, val):
        """
        SETTINGS_MAX_HEADER_LIST_SIZE only allows non-negative values.
        """
        s = h2.settings.Settings()

        if val >= 0:
            s.max_header_list_size = val
            s.acknowledge()
            assert s.max_header_list_size == val
        else:
            with pytest.raises(h2.exceptions.InvalidSettingsValueError) as e:
                s.max_header_list_size = val

            s.acknowledge()
            assert e.value.error_code == h2.errors.ErrorCodes.PROTOCOL_ERROR
            assert s.max_header_list_size is None

            with pytest.raises(h2.exceptions.InvalidSettingsValueError) as e:
                s[h2.settings.SettingCodes.MAX_HEADER_LIST_SIZE] = val

            s.acknowledge()
            assert e.value.error_code == h2.errors.ErrorCodes.PROTOCOL_ERROR

            with pytest.raises(KeyError):
                s[h2.settings.SettingCodes.MAX_HEADER_LIST_SIZE]

    @given(integers())
    def test_cannot_set_invalid_values_for_enable_connect_protocol(self, val):
        """
        SETTINGS_ENABLE_CONNECT_PROTOCOL only allows two values: 0, 1.
        """
        assume(val not in (0, 1))
        s = h2.settings.Settings()

        with pytest.raises(h2.exceptions.InvalidSettingsValueError) as e:
            s.enable_connect_protocol = val

        s.acknowledge()
        assert e.value.error_code == h2.errors.ErrorCodes.PROTOCOL_ERROR
        assert s.enable_connect_protocol == 0

        with pytest.raises(h2.exceptions.InvalidSettingsValueError) as e:
            s[h2.settings.SettingCodes.ENABLE_CONNECT_PROTOCOL] = val

        s.acknowledge()
        assert e.value.error_code == h2.errors.ErrorCodes.PROTOCOL_ERROR
        assert s[h2.settings.SettingCodes.ENABLE_CONNECT_PROTOCOL] == 0


class TestSettingsEquality(object):
    """
    A class defining tests for the standard implementation of == and != .
    """

    SettingsStrategy = builds(
        h2.settings.Settings,
        client=booleans(),
        initial_values=fixed_dictionaries({
            h2.settings.SettingCodes.HEADER_TABLE_SIZE:
                integers(0, 2**32 - 1),
            h2.settings.SettingCodes.ENABLE_PUSH: integers(0, 1),
            h2.settings.SettingCodes.INITIAL_WINDOW_SIZE:
                integers(0, 2**31 - 1),
            h2.settings.SettingCodes.MAX_FRAME_SIZE:
                integers(2**14, 2**24 - 1),
            h2.settings.SettingCodes.MAX_CONCURRENT_STREAMS:
                integers(0, 2**32 - 1),
            h2.settings.SettingCodes.MAX_HEADER_LIST_SIZE:
                integers(0, 2**32 - 1),
        })
    )

    @given(settings=SettingsStrategy)
    def test_equality_reflexive(self, settings):
        """
        An object compares equal to itself using the == operator and the !=
        operator.
        """
        assert (settings == settings)
        assert not (settings != settings)

    @given(settings=SettingsStrategy, o_settings=SettingsStrategy)
    def test_equality_multiple(self, settings, o_settings):
        """
        Two objects compare themselves using the == operator and the !=
        operator.
        """
        if settings == o_settings:
            assert settings == o_settings
            assert not (settings != o_settings)
        else:
            assert settings != o_settings
            assert not (settings == o_settings)

    @given(settings=SettingsStrategy)
    def test_another_type_equality(self, settings):
        """
        The object does not compare equal to an object of an unrelated type
        (which does not implement the comparison) using the == operator.
        """
        obj = object()
        assert (settings != obj)
        assert not (settings == obj)

    @given(settings=SettingsStrategy)
    def test_delegated_eq(self, settings):
        """
        The result of comparison is delegated to the right-hand operand if
        it is of an unrelated type.
        """
        class Delegate(object):
            def __eq__(self, other):
                return [self]

            def __ne__(self, other):
                return [self]

        delg = Delegate()
        assert (settings == delg) == [delg]
        assert (settings != delg) == [delg]
