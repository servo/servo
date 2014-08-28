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

Descriptions of individual config options can be translated to multiple
languages using gettext. Each option has associated with it a domain and locale
directory. By default, the domain is the section the option is in and the
locale directory is the "locale" directory beneath the directory containing the
module that defines it.

People implementing ConfigProvider instances are expected to define a complete
gettext .po and .mo file for the en-US locale. You can use the gettext-provided
msgfmt binary to perform this conversion. Generation of the original .po file
can be done via the write_pot() of ConfigSettings.
"""

from __future__ import unicode_literals

import collections
import gettext
import os
import sys

if sys.version_info[0] == 3:
    from configparser import RawConfigParser
    str_type = str
else:
    from ConfigParser import RawConfigParser
    str_type = basestring


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
        if not isinstance(value, str_type):
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
        if not isinstance(value, str_type):
            raise TypeError()

    @staticmethod
    def from_config(config, section, option):
        return config.get(section, option)


class AbsolutePathType(PathType):
    @staticmethod
    def validate(value):
        if not isinstance(value, str_type):
            raise TypeError()

        if not os.path.isabs(value):
            raise ValueError()


class RelativePathType(PathType):
    @staticmethod
    def validate(value):
        if not isinstance(value, str_type):
            raise TypeError()

        if os.path.isabs(value):
            raise ValueError()


class DefaultValue(object):
    pass


class ConfigProvider(object):
    """Abstract base class for an object providing config settings.

    Classes implementing this interface expose configurable settings. Settings
    are typically only relevant to that component itself. But, nothing says
    settings can't be shared by multiple components.
    """

    @classmethod
    def register_settings(cls):
        """Registers config settings.

        This is called automatically. Child classes should likely not touch it.
        See _register_settings() instead.
        """
        if hasattr(cls, '_settings_registered'):
            return

        cls._settings_registered = True

        cls.config_settings = {}

        ourdir = os.path.dirname(__file__)
        cls.config_settings_locale_directory = os.path.join(ourdir, 'locale')

        cls._register_settings()

    @classmethod
    def _register_settings(cls):
        """The actual implementation of register_settings().

        This is what child classes should implement. They should not touch
        register_settings().

        Implementations typically make 1 or more calls to _register_setting().
        """
        raise NotImplemented('%s must implement _register_settings.' %
            __name__)

    @classmethod
    def register_setting(cls, section, option, type_cls, default=DefaultValue,
        choices=None, domain=None):
        """Register a config setting with this type.

        This is a convenience method to populate available settings. It is
        typically called in the class's _register_settings() implementation.

        Each setting must have:

            section -- str section to which the setting belongs. This is how
                settings are grouped.

            option -- str id for the setting. This must be unique within the
                section it appears.

            type -- a ConfigType-derived type defining the type of the setting.

        Each setting has the following optional parameters:

            default -- The default value for the setting. If None (the default)
                there is no default.

            choices -- A set of values this setting can hold. Values not in
                this set are invalid.

            domain -- Translation domain for this setting. By default, the
                 domain is the same as the section name.
        """
        if not section in cls.config_settings:
            cls.config_settings[section] = {}

        if option in cls.config_settings[section]:
            raise Exception('Setting has already been registered: %s.%s' % (
                section, option))

        domain = domain if domain is not None else section

        meta = {
            'short': '%s.short' % option,
            'full': '%s.full' % option,
            'type_cls': type_cls,
            'domain': domain,
            'localedir': cls.config_settings_locale_directory,
        }

        if default != DefaultValue:
            meta['default'] = default

        if choices is not None:
            meta['choices'] = choices

        cls.config_settings[section][option] = meta


class ConfigSettings(collections.Mapping):
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

    class ConfigSection(collections.MutableMapping, object):
        """Represents an individual config section."""
        def __init__(self, config, name, settings):
            object.__setattr__(self, '_config', config)
            object.__setattr__(self, '_name', name)
            object.__setattr__(self, '_settings', settings)

        # MutableMapping interface
        def __len__(self):
            return len(self._settings)

        def __iter__(self):
            return iter(self._settings.keys())

        def __contains__(self, k):
            return k in self._settings

        def __getitem__(self, k):
            if k not in self._settings:
                raise KeyError('Option not registered with provider: %s' % k)

            meta = self._settings[k]

            if self._config.has_option(self._name, k):
                return meta['type_cls'].from_config(self._config, self._name, k)

            if not 'default' in meta:
                raise KeyError('No default value registered: %s' % k)

            return meta['default']

        def __setitem__(self, k, v):
            if k not in self._settings:
                raise KeyError('Option not registered with provider: %s' % k)

            meta = self._settings[k]

            meta['type_cls'].validate(v)

            if not self._config.has_section(self._name):
                self._config.add_section(self._name)

            self._config.set(self._name, k, meta['type_cls'].to_config(v))

        def __delitem__(self, k):
            self._config.remove_option(self._name, k)

            # Prune empty sections.
            if not len(self._config.options(self._name)):
                self._config.remove_section(self._name)

        def __getattr__(self, k):
            return self.__getitem__(k)

        def __setattr__(self, k, v):
            self.__setitem__(k, v)

        def __delattr__(self, k):
            self.__delitem__(k)


    def __init__(self):
        self._config = RawConfigParser()

        self._settings = {}
        self._sections = {}
        self._finalized = False
        self._loaded_filenames = set()

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
        self._loaded_filenames.update(set(filtered))
        for fp in fps:
            fp.close()

    def load_fps(self, fps):
        """Load config data by reading file objects."""

        for fp in fps:
            self._config.readfp(fp)

    def loaded_files(self):
        return self._loaded_filenames

    def write(self, fh):
        """Write the config to a file object."""
        self._config.write(fh)

    def validate(self):
        """Ensure that the current config passes validation.

        This is a generator of tuples describing any validation errors. The
        elements of the tuple are:

            (bool) True if error is fatal. False if just a warning.
            (str) Type of validation issue. Can be one of ('unknown-section',
                'missing-required', 'type-error')
        """

    def register_provider(self, provider):
        """Register a ConfigProvider with this settings interface."""

        if self._finalized:
            raise Exception('Providers cannot be registered after finalized.')

        provider.register_settings()

        for section_name, settings in provider.config_settings.items():
            section = self._settings.get(section_name, {})

            for k, v in settings.items():
                if k in section:
                    raise Exception('Setting already registered: %s.%s' %
                        section_name, k)

                section[k] = v

            self._settings[section_name] = section

    def write_pot(self, fh):
        """Write a pot gettext translation file."""

        for section in sorted(self):
            fh.write('# Section %s\n\n' % section)
            for option in sorted(self[section]):
                fh.write('msgid "%s.%s.short"\n' % (section, option))
                fh.write('msgstr ""\n\n')

                fh.write('msgid "%s.%s.full"\n' % (section, option))
                fh.write('msgstr ""\n\n')

            fh.write('# End of section %s\n\n' % section)

    def option_help(self, section, option):
        """Obtain the translated help messages for an option."""

        meta = self[section]._settings[option]

        # Providers should always have an en-US translation. If they don't,
        # they are coded wrong and this will raise.
        default = gettext.translation(meta['domain'], meta['localedir'],
            ['en-US'])

        t = gettext.translation(meta['domain'], meta['localedir'],
            fallback=True)
        t.add_fallback(default)

        short = t.ugettext('%s.%s.short' % (section, option))
        full = t.ugettext('%s.%s.full' % (section, option))

        return (short, full)

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
    def __getattr__(self, k):
        return self.__getitem__(k)
