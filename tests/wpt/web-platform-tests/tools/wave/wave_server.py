import os
import logging

from . import configuration_loader

from .network.http_handler import HttpHandler
from .network.api.sessions_api_handler import SessionsApiHandler
from .network.api.tests_api_handler import TestsApiHandler
from .network.api.results_api_handler import ResultsApiHandler
from .network.static_handler import StaticHandler

from .testing.sessions_manager import SessionsManager
from .testing.results_manager import ResultsManager
from .testing.tests_manager import TestsManager
from .testing.test_loader import TestLoader
from .testing.event_dispatcher import EventDispatcher


class WaveServer(object):
    def initialize(self,
                   tests,
                   configuration_file_path=None,
                   application_directory_path=None,
                   reports_enabled=None):
        if configuration_file_path is None:
            configuration_file_path = ""
        if application_directory_path is None:
            application_directory_path = ""
        if reports_enabled is None:
            reports_enabled = False

        logger = logging.getLogger("wave-server")

        logger.debug("Loading configuration ...")
        configuration = configuration_loader.load(configuration_file_path)

        # Initialize Managers
        event_dispatcher = EventDispatcher()
        sessions_manager = SessionsManager()
        results_manager = ResultsManager()
        tests_manager = TestsManager()
        test_loader = TestLoader()

        sessions_manager.initialize(
            test_loader=test_loader,
            event_dispatcher=event_dispatcher,
            tests_manager=tests_manager,
            results_directory=configuration["results_directory_path"],
            results_manager=results_manager
        )

        results_manager.initialize(
            results_directory_path=configuration["results_directory_path"],
            sessions_manager=sessions_manager,
            tests_manager=tests_manager,
            import_enabled=configuration["import_enabled"],
            reports_enabled=reports_enabled,
            persisting_interval=configuration["persisting_interval"]
        )

        tests_manager.initialize(
            test_loader,
            results_manager=results_manager,
            sessions_manager=sessions_manager,
            event_dispatcher=event_dispatcher
        )

        exclude_list_file_path = os.path.abspath("./excluded.json")
        include_list_file_path = os.path.abspath("./included.json")
        test_loader.initialize(
            exclude_list_file_path,
            include_list_file_path,
            results_manager=results_manager,
            api_titles=configuration["api_titles"]
        )

        test_loader.load_tests(tests)

        # Initialize HTTP handlers
        static_handler = StaticHandler(
            web_root=configuration["web_root"],
            http_port=configuration["wpt_port"],
            https_port=configuration["wpt_ssl_port"]
        )
        sessions_api_handler = SessionsApiHandler(
            sessions_manager=sessions_manager,
            results_manager=results_manager,
            event_dispatcher=event_dispatcher,
            web_root=configuration["web_root"]
        )
        tests_api_handler = TestsApiHandler(
            tests_manager=tests_manager,
            sessions_manager=sessions_manager,
            wpt_port=configuration["wpt_port"],
            wpt_ssl_port=configuration["wpt_ssl_port"],
            hostname=configuration["hostname"],
            web_root=configuration["web_root"],
            test_loader=test_loader
        )
        results_api_handler = ResultsApiHandler(
            results_manager,
            web_root=configuration["web_root"])

        # Initialize HTTP server
        http_handler = HttpHandler(
            static_handler=static_handler,
            sessions_api_handler=sessions_api_handler,
            tests_api_handler=tests_api_handler,
            results_api_handler=results_api_handler,
            http_port=configuration["wpt_port"],
            web_root=configuration["web_root"]
        )
        self.handle_request = http_handler.handle_request
