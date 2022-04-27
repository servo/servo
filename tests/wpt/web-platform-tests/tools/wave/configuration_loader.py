import json
import os

from tools.wpt import wpt

DEFAULT_CONFIGURATION_FILE_PATH = os.path.join(wpt.localpaths.repo_root, "./tools/wave/config.default.json")


def load(configuration_file_path):
    configuration = {}
    if configuration_file_path:
        configuration = load_configuration_file(configuration_file_path)
    default_configuration = load_configuration_file(
        DEFAULT_CONFIGURATION_FILE_PATH)

    configuration["wpt_port"] = configuration.get(
        "ports", default_configuration["ports"]).get(
        "http", default_configuration["ports"]["http"])[0]
    configuration["wpt_ssl_port"] = configuration.get(
        "ports", default_configuration["ports"]).get(
        "https", default_configuration["ports"]["https"])[0]

    web_root = configuration.get(
        "wave", default_configuration["wave"]).get(
        "web_root", default_configuration["wave"]["web_root"])
    if not web_root.startswith("/"):
        web_root = "/" + web_root
    if not web_root.endswith("/"):
        web_root += "/"
    configuration["web_root"] = web_root

    configuration["results_directory_path"] = configuration.get(
        "wave", default_configuration["wave"]).get(
        "results", default_configuration["wave"]["results"])

    configuration["timeouts"] = {}
    configuration["timeouts"]["automatic"] = configuration.get(
        "wave", default_configuration["wave"]).get(
        "timeouts", default_configuration["wave"]["timeouts"]).get(
        "automatic", default_configuration["wave"]["timeouts"]["automatic"])
    configuration["timeouts"]["manual"] = configuration.get(
        "wave", default_configuration["wave"]).get(
        "timeouts", default_configuration["wave"]["timeouts"]).get(
        "manual", default_configuration["wave"]["timeouts"]["manual"])

    configuration["hostname"] = configuration.get(
        "browser_host", default_configuration["browser_host"])

    configuration["import_results_enabled"] = configuration.get(
        "wave", default_configuration["wave"]).get(
        "enable_import_results",
        default_configuration["wave"]["enable_import_results"])

    configuration["read_sessions_enabled"] = configuration.get(
        "wave", default_configuration["wave"]).get(
        "enable_read_sessions",
        default_configuration["wave"]["enable_read_sessions"])

    configuration["persisting_interval"] = configuration.get(
        "wave", default_configuration["wave"]).get(
        "persisting_interval", default_configuration["wave"]["persisting_interval"])

    configuration["event_cache_duration"] = configuration.get(
        "wave", default_configuration["wave"]).get(
        "event_cache_duration", default_configuration["wave"]["event_cache_duration"])

    configuration["tests_directory_path"] = os.getcwd()

    configuration["manifest_file_path"] = os.path.join(
        os.getcwd(), "MANIFEST.json")

    configuration["api_titles"] = configuration.get(
        "wave", default_configuration["wave"]).get(
        "api_titles", default_configuration["wave"]["api_titles"])

    configuration["enable_test_type_selection"] = configuration.get(
        "wave", default_configuration["wave"]).get(
        "enable_test_type_selection", default_configuration["wave"]["enable_test_type_selection"])

    configuration["enable_test_file_selection"] = configuration.get(
        "wave", default_configuration["wave"]).get(
        "enable_test_file_selection", default_configuration["wave"]["enable_test_file_selection"])

    return configuration


def load_configuration_file(path):
    if not os.path.isfile(path):
        return {}

    configuration = None
    with open(path) as configuration_file:
        configuration_file_content = configuration_file.read()
        configuration = json.loads(configuration_file_content)
    return configuration
