# mypy: allow-untyped-defs

from ua_parser import user_agent_parser


def parse_user_agent(user_agent_string):
    user_agent = user_agent_parser.ParseUserAgent(user_agent_string)

    name = user_agent["family"]
    version = "0"

    if user_agent["major"] is not None:
        version = user_agent["major"]

    if user_agent["minor"] is not None:
        version = version + "." + user_agent["minor"]

    if user_agent["patch"] is not None:
        version = version + "." + user_agent["patch"]

    return {
        "name": name,
        "version": version
    }


def abbreviate_browser_name(name):
    short_names = {
        "Chrome": "Ch",
        "Chrome Mobile WebView": "Ch",
        "Chromium": "Cm",
        "WebKit": "Wk",
        "Safari": "Sf",
        "Firefox": "FF",
        "IE": "IE",
        "Edge": "Ed",
        "Opera": "Op"
    }

    if name in short_names:
        return short_names[name]
    else:
        return "Xx"
