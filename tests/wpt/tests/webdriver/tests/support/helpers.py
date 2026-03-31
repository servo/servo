import base64
import collections
import os
from urllib.parse import urlparse


def deep_update(source, overrides):
    """
    Update a nested dictionary or similar mapping.
    Modify ``source`` in place.
    """
    for key, value in overrides.items():
        if isinstance(value, collections.abc.Mapping) and value:
            source[key] = deep_update(source.get(key, {}), value)
        elif isinstance(value, list) and isinstance(source.get(key), list) and value:
            # Concatenate lists, ensuring all elements are kept without duplicates
            source[key] = list(dict.fromkeys(source[key] + value))
        else:
            source[key] = value

    return source


def filter_dict(source, d):
    """Filter `source` dict to only contain same keys as `d` dict.

    :param source: dictionary to filter.
    :param d: dictionary whose keys determine the filtering.
    """
    return {k: source[k] for k in d.keys()}


def filter_supported_key_events(all_events, expected):
    events = [filter_dict(e, expected[0]) for e in all_events]
    if len(events) > 0 and events[0]["code"] is None:
        # Remove 'code' entry if browser doesn't support it
        expected = [filter_dict(e, {"key": "", "type": ""}) for e in expected]
        events = [filter_dict(e, expected[0]) for e in events]

    return (events, expected)


def get_origin_from_url(url):
    parsed_uri = urlparse(url)
    return '{uri.scheme}://{uri.netloc}'.format(uri=parsed_uri)


def get_extension_path(filename):
    return os.path.join(
        os.path.abspath(os.path.dirname(__file__)), "webextensions", filename
    )


def get_base64_for_extension_file(filename):
    with open(
        get_extension_path(filename),
        "rb",
    ) as file:
        return base64.b64encode(file.read()).decode("utf-8")


def is_wayland():
    # We don't use mozinfo.display here to make sure it also
    # works upstream in wpt Github repo.
    return os.environ.get("WAYLAND_DISPLAY", "") != ""
