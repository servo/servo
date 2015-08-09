# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

from ..reader import LogHandler

class BaseFormatter(LogHandler):
    """Base class for implementing non-trivial formatters.

    Subclasses are expected to provide a method for each action type they
    wish to handle, each taking a single argument for the test data.
    For example a trivial subclass that just produces the id of each test as
    it starts might be::

      class StartIdFormatter(BaseFormatter);
          def test_start(data):
              #For simplicity in the example pretend the id is always a string
              return data["test"]
    """
