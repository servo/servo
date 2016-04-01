import ConfigParser
import json
import os
import sys
import unittest

from network import get_lan_ip

repo_root = os.path.abspath(os.path.join(__file__, "../.."))
sys.path.insert(1, os.path.join(repo_root, "tools", "webdriver"))
sys.path.insert(1, os.path.join(repo_root, "tools", "wptserve"))
from wptserve import server
from selenium import webdriver


class WebDriverBaseTest(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.driver = create_driver()

        cls.webserver = server.WebTestHttpd(host=get_lan_ip())
        cls.webserver.start()
        cls.webserver.where_is = cls.webserver.get_url

    @classmethod
    def tearDownClass(cls):
        cls.webserver.stop()
        if cls.driver:
            cls.driver.quit()


def create_driver():
    config = ConfigParser.ConfigParser()
    config.read('webdriver.cfg')
    section = os.environ.get("WD_BROWSER", 'firefox')
    if config.has_option(section, 'url'):
        url = config.get(section, "url")
    else:
        url = 'http://127.0.0.1:4444/wd/hub'
    capabilities = None
    if config.has_option(section, 'capabilities'):
        try:
            capabilities = json.loads(config.get(section, "capabilities"))
        except:
            pass
    mode = 'compatibility'
    if config.has_option(section, 'mode'):
        mode = config.get(section, 'mode')
    if section == 'firefox':
        driver = webdriver.Firefox()
    elif section == 'chrome':
        driver = webdriver.Chrome()
    elif section == 'edge':
        driver = webdriver.Remote()
    elif section == 'ie':
        driver = webdriver.Ie()
    elif section == 'selendroid':
        driver = webdriver.Android()

    return driver
