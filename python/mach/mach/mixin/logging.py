# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

from __future__ import absolute_import, unicode_literals

import logging


class LoggingMixin(object):
    """Provides functionality to control logging."""

    def populate_logger(self, name=None):
        """Ensure this class instance has a logger associated with it.

        Users of this mixin that call log() will need to ensure self._logger is
        a logging.Logger instance before they call log(). This function ensures
        self._logger is defined by populating it if it isn't.
        """
        if hasattr(self, '_logger'):
            return

        if name is None:
            name = '.'.join([self.__module__, self.__class__.__name__])

        self._logger = logging.getLogger(name)

    def log(self, level, action, params, format_str):
        """Log a structured log event.

        A structured log event consists of a logging level, a string action, a
        dictionary of attributes, and a formatting string.

        The logging level is one of the logging.* constants, such as
        logging.INFO.

        The action string is essentially the enumeration of the event. Each
        different type of logged event should have a different action.

        The params dict is the metadata constituting the logged event.

        The formatting string is used to convert the structured message back to
        human-readable format. Conversion back to human-readable form is
        performed by calling format() on this string, feeding into it the dict
        of attributes constituting the event.

        Example Usage
        -------------

        self.log(logging.DEBUG, 'login', {'username': 'johndoe'},
            'User login: {username}')
        """
        self._logger.log(level, format_str,
            extra={'action': action, 'params': params})

