import os
import sys
import unittest

sys.path.insert(1, os.path.abspath(os.path.join(__file__, "../..")))
import base_test
from selenium.common import exceptions


class CookieTest(base_test.WebDriverBaseTest):
    def setUp(self):
        self.driver.get(self.webserver.where_is("cookie/res/cookie_container.html"))

    def test_can_create_a_well_formed_cookie( self ):
        self.driver.add_cookie({'name': 'foo', 'value': 'bar'})

    def test_cookies_should_allow_secure_to_be_set( self ):
        name = 'foo'
        self.driver.add_cookie({'name': name,
                                'value': 'bar',
                                'path': '/',
                                'secure': (True)})
        self.assertTrue(self.driver.get_cookie(name)[0]['secure'])

    def test_secure_defaults_to_false( self ):
        name = 'foo'
        value = 'bar'

        self.driver.add_cookie({ 'name': name,
                                 'value': value})

        self.assertFalse(self.driver.get_cookie(name)[0]['secure'])

    def test_should_throw_an_exception_when_semicolon_exists_in_the_cookie_attribute(self):
        invalid_name = 'foo;bar'
        try:
            self.driver.add_cookie({'name': invalid_name, 'value': 'foobar'})
            self.fail( 'should have thrown exceptions.' )

        except exceptions.UnableToSetCookieException:
            pass
        except exceptions.InvalidCookieDomainException:
            pass

    def test_should_throw_an_exception_the_name_is_null(self):
        try:
            self.driver.add_cookie({'name': None, 'value': 'foobar'})
            self.fail( 'should have thrown exceptions.' )

        except exceptions.UnableToSetCookieException:
            pass
        except exceptions.InvalidCookieDomainException:
            pass


if __name__ == '__main__':
    unittest.main()
