# mypy: allow-untyped-defs

import json

import webdriver


"""WebDriver wire protocol codecs."""


class Encoder(json.JSONEncoder):
    def __init__(self, *args, **kwargs):
        kwargs.pop("session")
        super().__init__(*args, **kwargs)

    def default(self, obj):
        if isinstance(obj, (list, tuple)):
            return [self.default(x) for x in obj]
        elif isinstance(obj, webdriver.WebElement):
            return {webdriver.WebElement.identifier: obj.id}
        elif isinstance(obj, webdriver.WebFrame):
            return {webdriver.WebFrame.identifier: obj.id}
        elif isinstance(obj, webdriver.ShadowRoot):
            return {webdriver.ShadowRoot.identifier: obj.id}
        elif isinstance(obj, webdriver.WebWindow):
            return {webdriver.WebWindow.identifier: obj.id}
        # Support for arguments received via BiDi.
        # https://github.com/web-platform-tests/rfcs/blob/master/rfcs/testdriver_bidi.md
        elif isinstance(obj, webdriver.bidi.protocol.BidiValue):
            return obj.to_classic_protocol_value()

        return super().default(obj)


class Decoder(json.JSONDecoder):
    def __init__(self, *args, **kwargs):
        self.session = kwargs.pop("session")
        super().__init__(
            object_hook=self.object_hook, *args, **kwargs)

    def object_hook(self, payload):
        if isinstance(payload, (list, tuple)):
            return [self.object_hook(x) for x in payload]
        elif isinstance(payload, dict) and webdriver.WebElement.identifier in payload:
            return webdriver.WebElement.from_json(payload, self.session)
        elif isinstance(payload, dict) and webdriver.WebFrame.identifier in payload:
            return webdriver.WebFrame.from_json(payload, self.session)
        elif isinstance(payload, dict) and webdriver.ShadowRoot.identifier in payload:
            return webdriver.ShadowRoot.from_json(payload, self.session)
        elif isinstance(payload, dict) and webdriver.WebWindow.identifier in payload:
            return webdriver.WebWindow.from_json(payload, self.session)
        elif isinstance(payload, dict):
            return {k: self.object_hook(v) for k, v in payload.items()}
        return payload
