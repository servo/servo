# mypy: allow-untyped-defs

class Device:
    def __init__(self, token, user_agent, name, last_active):
        self.token = token
        self.user_agent = user_agent
        self.name = name
        self.last_active = last_active
