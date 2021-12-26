# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this file,
# You can obtain one at http://mozilla.org/MPL/2.0/.

r"""
This file defines classes for representing config data/settings.

Config data is modeled as key-value pairs. Keys are grouped together into named
sections. Individual config settings (options) have metadata associated with
them. This metadata includes type, default value, valid values, etc.

The main interface to config data is the ConfigSettings class. 1 or more
ConfigProvider classes are associated with ConfigSettings and define what
settings are available.
"""

from __future__ import absolute_import, unicode_literals

import collections
import os
import sys
import six
from functools import wraps
from six.moves.configparser import RawConfigParser, NoSectionError
from six import string_types


class ConfigException(Exception):
    pass


class ConfigType(object):
    """Abstract base class for config values."""

    @staticmethod
    def validate(value):
        """Validates a Python value conforms to this type.

        Raises a TypeError or ValueError if it doesn't conform. Does not do
        anything if the value is valid.
        """

    @staticmethod
    def from_config(config, section, option):
        """Obtain the value of this type from a RawConfigParser.

        Receives a RawConfigParser instance, a str section name, and the str
        option in that section to retrieve.

        The implementation may assume the option exists in the RawConfigParser
        instance.

        Implementations are not expected to validate the value. But, they
        should return the appropriate Python type.
        """

    @staticmethod
    def to_config(value):
        return value


class StringType(ConfigType):
    @staticmethod
    def validate(value):
        if not isinstance(value, string_types):
            raise TypeError()

    @staticmethod
    def from_config(config, section, option):
        return config.get(section, option)


class BooleanType(ConfigType):
    @staticmethod
    def validate(value):
        if not isinstance(value, bool):
            raise TypeError()

    @staticmethod
    def from_config(config, section, option):
        return config.getboolean(section, option)

    @staticmethod
    def to_config(value):
        return 'true' if value else 'false'


class IntegerType(ConfigType):
    @staticmethod
    def validate(value):
        if not isinstance(value, int):
            raise TypeError()

    @staticmethod
    def from_config(config, section, option):
        return config.getint(section, option)


class PositiveIntegerType(IntegerType):
    @staticmethod
    def validate(value):
        if not isinstance(value, int):
            raise TypeError()

        if value < 0:
            raise ValueError()


class PathType(StringType):
    @staticmethod
    def validate(value):
        if not isinstance(value, string_types):
            raise TypeError()

    @staticmethod
    def from_config(config, section, option):
        return config.get(section, option)


TYPE_CLASSES = {
    'string': StringType,
    'boolean': BooleanType,
    'int': IntegerType,
    'pos_int': PositiveIntegerType,
    'path': PathType,
}


class DefaultValue(object):
    pass


def reraise_attribute_error(func):
    """Used to make sure __getattr__ wrappers around __getitem__
    raise AttributeError instead of KeyError.
    """
    @wraps(func)
    def _(*args, **kwargs):
        try:
            return func(*args, **kwargs)
        except KeyError:
            exc_class, exc, tb = sys.exc_info()
            six.reraise(AttributeError().__class__, exc, tb)
    return _


class ConfigSettings(collections.abc.Mapping):
    """Interface for configuration settings.

    This is the main interface to the configuration.

    A configuration is a collection of sections. Each section contains
    key-value pairs.

    When an instance is created, the caller first registers ConfigProvider
    instances with it. This tells the ConfigSettings what individual settings
    are available and defines extra metadata associated with those settings.
    This is used for validation, etc.

    Once ConfigProvider instances are registered, a config is populated. It can
    be loaded from files or populated by hand.

    ConfigSettings instances are accessed like dictionaries or by using
    attributes. e.g. the section "foo" is accessed through either
    settings.foo or settings['foo'].

    Sections are modeled by the ConfigSection class which is defined inside
    this one. They look just like dicts or classes with attributes. To access
    the "bar" option in the "foo" section:

        value = settings.foo.bar
        value = settings['foo']['bar']
        value = settings.foo['bar']

    Assignment is similar:

        settings.foo.bar = value
        settings['foo']['bar'] = value
        settings['foo'].bar = value

    You can even delete user-assigned values:

        del settings.foo.bar
        del settings['foo']['bar']

    If there is a default, it will be returned.

    When settings are mutated, they are validated against the registered
    providers. Setting unknown settings or setting values to illegal values
    will result in exceptions being raised.
    """

    class ConfigSection(collections.abc.MutableMapping, object):
        """Represents an individual config section."""
        def __init__(self, config, name, settings):
            object.__setattr__(self, '_config', config)
            object.__setattr__(self, '_name', name)
            object.__setattr__(self, '_settings', settings)

            wildcard = any(s == '*' for s in self._settings)
            object.__setattr__(self, '_wildcard', wildcard)

        @property
        def options(self):
            try:
                return self._config.options(self._name)
            except NoSectionError:
                return []

        def get_meta(self, option):
            if option in self._settings:
                return self._settings[option]
            if self._wildcard:
                return self._settings['*']
            raise KeyError('Option not registered with provider: %s' % option)

        def _validate(self, option, value):
            meta = self.get_meta(option)
            meta['type_cls'].validate(value)

            if 'choices' in meta and value not in meta['choices']:
                raise ValueError("Value '%s' must be one of: %s" % (
                                 value, ', '.join(sorted(meta['choices']))))

        # MutableMapping interface
        def __len__(self):
            return len(self.options)

        def __iter__(self):
            return iter(self.options)

        def __contains__(self, k):
            return self._config.has_option(self._name, k)

        def __getitem__(self, k):
            meta = self.get_meta(k)

            if self._config.has_option(self._name, k):
                v = meta['type_cls'].from_config(self._config, self._name, k)
            else:
                v = meta.get('default', DefaultValue)

            if v == DefaultValue:
                raise KeyError('No default value registered: %s' % k)

            self._validate(k, v)
            return v

        def __setitem__(self, k, v):
            self._validate(k, v)
            meta = self.get_meta(k)

            if not self._config.has_section(self._name):
                self._config.add_section(self._name)

            self._config.set(self._name, k, meta['type_cls'].to_config(v))

        def __delitem__(self, k):
            self._config.remove_option(self._name, k)

            # Prune empty sections.
            if not len(self._config.options(self._name)):
                self._config.remove_section(self._name)

        @reraise_attribute_error
        def __getattr__(self, k):
            return self.__getitem__(k)

        @reraise_attribute_error
        def __setattr__(self, k, v):
            self.__setitem__(k, v)

        @reraise_attribute_error
        def __delattr__(self, k):
            self.__delitem__(k)

    def __init__(self):
        self._config = RawConfigParser()
        self._config.optionxform = str

        self._settings = {}
        self._sections = {}
        self._finalized = False
        self.loaded_files = set()

    def load_file(self, filename):
        self.load_files([filename])

    def load_files(self, filenames):
        """Load a config from files specified by their paths.

        Files are loaded in the order given. Subsequent files will overwrite
        values from previous files. If a file does not exist, it will be
        ignored.
        """
        filtered = [f for f in filenames if os.path.exists(f)]

        fps = [open(f, 'rt') for f in filtered]
        self.load_fps(fps)
        self.loaded_files.update(set(filtered))
        for fp in fps:
            fp.close()

    def load_fps(self, fps):
        """Load config data by reading file objects."""

        for fp in fps:
            self._config.readfp(fp)

    def write(self, fh):
        """Write the config to a file object."""
        self._config.write(fh)

    @classmethod
    def _format_metadata(cls, provider, section, option, type_cls, description,
                         default=DefaultValue, extra=None):
        """Formats and returns the metadata for a setting.

        Each setting must have:

            section -- str section to which the setting belongs. This is how
                settings are grouped.

            option -- str id for the setting. This must be unique within the
                section it appears.

            type -- a ConfigType-derived type defining the type of the setting.

            description -- str describing how to use the setting and where it
                applies.

        Each setting has the following optional parameters:

            default -- The default value for the setting. If None (the default)
                there is no default.

            extra -- A dict of additional key/value pairs to add to the
                setting metadata.
        """
        if isinstance(type_cls, string_types):
            type_cls = TYPE_CLASSES[type_cls]

        meta = {
            'description': description,
            'type_cls': type_cls,
        }

        if default != DefaultValue:
            meta['default'] = default

        if extra:
            meta.update(extra)

        return meta

    def register_provider(self, provider):
        """Register a SettingsProvider with this settings interface."""

        if self._finalized:
            raise ConfigException('Providers cannot be registered after finalized.')

        settings = provider.config_settings
        if callable(settings):
            settings = settings()

        config_settings = collections.defaultdict(dict)
        for setting in settings:
            section, option = setting[0].split('.')

            if option in config_settings[section]:
                raise ConfigException('Setting has already been registered: %s.%s' % (
                                section, option))

            meta = self._format_metadata(provider, section, option, *setting[1:])
            config_settings[section][option] = meta

        for section_name, settings in config_settings.items():
            section = self._settings.get(section_name, {})

            for k, v in settings.items():
                if k in section:
                    raise ConfigException('Setting already registered: %s.%s' %
                                          (section_name, k))

                section[k] = v

            self._settings[section_name] = section

    def _finalize(self):
        if self._finalized:
            return

        for section, settings in self._settings.items():
            s = ConfigSettings.ConfigSection(self._config, section, settings)
            self._sections[section] = s

        self._finalized = True

    # Mapping interface.
    def __len__(self):
        return len(self._settings)

    def __iter__(self):
        self._finalize()

        return iter(self._sections.keys())

    def __contains__(self, k):
        return k in self._settings

    def __getitem__(self, k):
        self._finalize()

        return self._sections[k]

    # Allow attribute access because it looks nice.
    @reraise_attribute_error
    def __getattr__(self, k):
        return self.__getitem__(k)
