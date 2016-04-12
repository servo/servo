# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this file,
# You can obtain one at http://mozilla.org/MPL/2.0/.

class ServoExtensionCommands(object):
    def __init__(self, session):
        self.session = session

    @command
    def get_prefs(self, *prefs):
        body = {"prefs": list(prefs)}
        return self.session.send_command("POST", "servo/prefs/get", body)

    @command
    def set_prefs(self, prefs):
        body = {"prefs": prefs}
        return self.session.send_command("POST", "servo/prefs/set", body)

    @command
    def reset_prefs(self, *prefs):
        body = {"prefs": list(prefs)}
        return self.session.send_command("POST", "servo/prefs/reset", body)
