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
