import json

import webdriver


"""WebDriver wire protocol codecs."""


class Encoder(json.JSONEncoder):
    def __init__(self, *args, **kwargs):
        kwargs.pop("session")
        super(Encoder, self).__init__(*args, **kwargs)

    def default(self, obj):
        if isinstance(obj, (list, tuple)):
            return [self.default(x) for x in obj]
        elif isinstance(obj, webdriver.Element):
            return {webdriver.Element.identifier: obj.id}
        elif isinstance(obj, webdriver.Frame):
            return {webdriver.Frame.identifier: obj.id}
        elif isinstance(obj, webdriver.Window):
            return {webdriver.Frame.identifier: obj.id}
        return super(Encoder, self).default(obj)


class Decoder(json.JSONDecoder):
    def __init__(self, *args, **kwargs):
        self.session = kwargs.pop("session")
        super(Decoder, self).__init__(
            object_hook=self.object_hook, *args, **kwargs)

    def object_hook(self, payload):
        if isinstance(payload, (list, tuple)):
            return [self.object_hook(x) for x in payload]
        elif isinstance(payload, dict) and webdriver.Element.identifier in payload:
            return webdriver.Element.from_json(payload, self.session)
        elif isinstance(payload, dict) and webdriver.Frame.identifier in payload:
            return webdriver.Frame.from_json(payload, self.session)
        elif isinstance(payload, dict) and webdriver.Window.identifier in payload:
            return webdriver.Window.from_json(payload, self.session)
        elif isinstance(payload, dict):
            return {k: self.object_hook(v) for k, v in payload.iteritems()}
        return payload
