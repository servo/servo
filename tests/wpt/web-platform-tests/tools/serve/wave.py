# The ./wpt serve-wave command is broken, so mypy errors are ignored instead of
# making untestable changes to the problematic imports.
# See https://github.com/web-platform-tests/wpt/issues/29024.
# mypy: ignore-errors

import subprocess
from manifest import manifest
import localpaths
import os

try:
    from serve import serve
except ImportError:
    import serve

from tools.wpt import wpt


class WaveHandler:
    def __init__(self, server):
        self.server = server

    def __call__(self, request, response):
        self.server.handle_request(request, response)


def get_route_builder_func(report):
    def get_route_builder(logger, aliases, config):
        wave_cfg = None
        if config is not None and "wave" in config:
            wave_cfg = config["wave"]
        builder = serve.get_route_builder(logger, aliases, config)
        logger.debug("Loading manifest ...")
        data = load_manifest()
        from ..wave.wave_server import WaveServer
        wave_server = WaveServer()
        wave_server.initialize(
            configuration_file_path=os.path.abspath("./config.json"),
            reports_enabled=report,
            tests=data["items"])

        web_root = "wave"
        if wave_cfg is not None and "web_root" in wave_cfg:
            web_root = wave_cfg["web_root"]
        if not web_root.startswith("/"):
            web_root = "/" + web_root

        wave_handler = WaveHandler(wave_server)
        builder.add_handler("*", web_root + "*", wave_handler)
        # serving wave specifc testharnessreport.js
        file_path = os.path.join(wpt.localpaths.repo_root, "tools/wave/resources/testharnessreport.js")
        builder.add_static(
            file_path,
            {},
            "text/javascript;charset=utf8",
            "/resources/testharnessreport.js")

        return builder
    return get_route_builder


class ConfigBuilder(serve.ConfigBuilder):
    _default = serve.ConfigBuilder._default
    _default.update({
        "wave": {  # wave specific configuration parameters
            "results": "./results",
            "timeouts": {
                "automatic": 60000,
                "manual": 300000
            },
            "enable_results_import": False,
            "web_root": "/_wave",
            "persisting_interval": 20,
            "api_titles": []
        }
    })


def get_parser():
    parser = serve.get_parser()
    # Added wave specific arguments
    parser.add_argument("--report", action="store_true", dest="report",
                        help="Flag for enabling the WPTReporting server")
    return parser


def run(venv=None, **kwargs):
    if venv is not None:
        venv.start()
    else:
        raise Exception("Missing virtualenv for serve-wave.")

    if kwargs['report'] is True:
        if not is_wptreport_installed():
            raise Exception("wptreport is not installed. Please install it from https://github.com/w3c/wptreport")

    serve.run(config_cls=ConfigBuilder,
              route_builder=get_route_builder_func(kwargs["report"]),
              log_handlers=None,
              **kwargs)


# execute wptreport version check
def is_wptreport_installed():
    try:
        subprocess.check_output(["wptreport", "--help"])
        return True
    except Exception:
        return False


def load_manifest():
    root = localpaths.repo_root
    path = os.path.join(root, "MANIFEST.json")
    manifest_file = manifest.load_and_update(root, path, "/", parallel=False)

    supported_types = ["testharness", "manual"]
    data = {"items": {},
            "url_base": "/"}
    for item_type in supported_types:
        data["items"][item_type] = {}
    for item_type, path, tests in manifest_file.itertypes(*supported_types):
        tests_data = []
        for item in tests:
            test_data = [item.url[1:]]
            if item_type == "reftest":
                test_data.append(item.references)
            test_data.append({})
            if item_type != "manual":
                test_data[-1]["timeout"] = item.timeout
            tests_data.append(test_data)
        assert path not in data["items"][item_type]
        data["items"][item_type][path] = tests_data
    return data
