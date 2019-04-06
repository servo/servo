import os, sys, base64
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import subresource

def generate_payload(request, server_data):
    data = ('{"headers": %(headers)s}') % server_data
    if "id" in request.GET:
        request.server.stash.put(request.GET["id"], data)
    # Simple base64 encoded .tff font
    return base64.decodestring("AAEAAAANAIAAAwBQRkZUTU6u6MkAAAXcAAAAHE9TLzJWYW"
                               "QKAAABWAAAAFZjbWFwAA8D7wAAAcAAAAFCY3Z0IAAhAnkA"
                               "AAMEAAAABGdhc3D//wADAAAF1AAAAAhnbHlmCC6aTwAAAx"
                               "QAAACMaGVhZO8ooBcAAADcAAAANmhoZWEIkAV9AAABFAAA"
                               "ACRobXR4EZQAhQAAAbAAAAAQbG9jYQBwAFQAAAMIAAAACm"
                               "1heHAASQA9AAABOAAAACBuYW1lehAVOgAAA6AAAAIHcG9z"
                               "dP+uADUAAAWoAAAAKgABAAAAAQAAMhPyuV8PPPUACwPoAA"
                               "AAAMU4Lm0AAAAAxTgubQAh/5wFeAK8AAAACAACAAAAAAAA"
                               "AAEAAAK8/5wAWgXcAAAAAAV4AAEAAAAAAAAAAAAAAAAAAA"
                               "AEAAEAAAAEAAwAAwAAAAAAAgAAAAEAAQAAAEAALgAAAAAA"
                               "AQXcAfQABQAAAooCvAAAAIwCigK8AAAB4AAxAQIAAAIABg"
                               "kAAAAAAAAAAAABAAAAAAAAAAAAAAAAUGZFZABAAEEAQQMg"
                               "/zgAWgK8AGQAAAABAAAAAAAABdwAIQAAAAAF3AAABdwAZA"
                               "AAAAMAAAADAAAAHAABAAAAAAA8AAMAAQAAABwABAAgAAAA"
                               "BAAEAAEAAABB//8AAABB////wgABAAAAAAAAAQYAAAEAAA"
                               "AAAAAAAQIAAAACAAAAAAAAAAAAAAAAAAAAAQAAAAAAAAAA"
                               "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAwAAAAAAAA"
                               "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA"
                               "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA"
                               "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA"
                               "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA"
                               "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA"
                               "AAAAAAAAAAAAAAAAAAAhAnkAAAAqACoAKgBGAAAAAgAhAA"
                               "ABKgKaAAMABwAusQEALzyyBwQA7TKxBgXcPLIDAgDtMgCx"
                               "AwAvPLIFBADtMrIHBgH8PLIBAgDtMjMRIREnMxEjIQEJ6M"
                               "fHApr9ZiECWAAAAwBk/5wFeAK8AAMABwALAAABNSEVATUh"
                               "FQE1IRUB9AH0/UQDhPu0BRQB9MjI/tTIyP7UyMgAAAAAAA"
                               "4ArgABAAAAAAAAACYATgABAAAAAAABAAUAgQABAAAAAAAC"
                               "AAYAlQABAAAAAAADACEA4AABAAAAAAAEAAUBDgABAAAAAA"
                               "AFABABNgABAAAAAAAGAAUBUwADAAEECQAAAEwAAAADAAEE"
                               "CQABAAoAdQADAAEECQACAAwAhwADAAEECQADAEIAnAADAA"
                               "EECQAEAAoBAgADAAEECQAFACABFAADAAEECQAGAAoBRwBD"
                               "AG8AcAB5AHIAaQBnAGgAdAAgACgAYwApACAAMgAwADAAOA"
                               "AgAE0AbwB6AGkAbABsAGEAIABDAG8AcgBwAG8AcgBhAHQA"
                               "aQBvAG4AAENvcHlyaWdodCAoYykgMjAwOCBNb3ppbGxhIE"
                               "NvcnBvcmF0aW9uAABNAGEAcgBrAEEAAE1hcmtBAABNAGUA"
                               "ZABpAHUAbQAATWVkaXVtAABGAG8AbgB0AEYAbwByAGcAZQ"
                               "AgADIALgAwACAAOgAgAE0AYQByAGsAQQAgADoAIAA1AC0A"
                               "MQAxAC0AMgAwADAAOAAARm9udEZvcmdlIDIuMCA6IE1hcm"
                               "tBIDogNS0xMS0yMDA4AABNAGEAcgBrAEEAAE1hcmtBAABW"
                               "AGUAcgBzAGkAbwBuACAAMAAwADEALgAwADAAMAAgAABWZX"
                               "JzaW9uIDAwMS4wMDAgAABNAGEAcgBrAEEAAE1hcmtBAAAA"
                               "AgAAAAAAAP+DADIAAAABAAAAAAAAAAAAAAAAAAAAAAAEAA"
                               "AAAQACACQAAAAAAAH//wACAAAAAQAAAADEPovuAAAAAMU4"
                               "Lm0AAAAAxTgubQ==");

def generate_report_headers_payload(request, server_data):
    stashed_data = request.server.stash.take(request.GET["id"])
    return stashed_data

def main(request, response):
    handler = lambda data: generate_payload(request, data)
    content_type = 'application/x-font-truetype'

    if "report-headers" in request.GET:
        handler = lambda data: generate_report_headers_payload(request, data)
        content_type = 'application/json'

    subresource.respond(request,
                        response,
                        payload_generator = handler,
                        content_type = content_type,
                        access_control_allow_origin = "*")
