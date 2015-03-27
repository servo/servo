"""WebDriver element location functionality."""

class SearchContext(object):
    """Abstract class that provides the core element location functionality."""

    def find_element_by_css(self, selector):
        """Find the first element matching a css selector."""
        return self._find_element('css selector', selector)

    def find_elements_by_css(self, selector):
        """Find all elements matching a css selector."""
        return self._find_elements('css selector', selector)

    def find_element_by_link_text(self, text):
        """Find the first link with the given text."""
        return self._find_element('link text', text)

    def find_elements_by_link_text(self, text):
        """Find all links with the given text."""
        return self._find_elements('link text', text)

    def find_element_by_partial_link_text(self, text):
        """Find the first link containing the given text."""
        return self._find_element('partial link text', text)

    def find_elements_by_partial_link_text(self, text):
        """Find all links containing the given text."""
        return self._find_elements('partial link text', text)

    def find_element_by_xpath(self, xpath):
        """Find the first element matching the xpath."""
        return self._find_element('xpath', xpath)

    def find_elements_by_xpath(self, xpath):
        """Find all elements matching the xpath."""
        return self._find_elements('xpath', xpath)

    def _find_element(self, strategy, value):
        return self.execute('POST',
                            '/element',
                            'findElement',
                            self._get_locator(strategy, value))

    def _find_elements(self, strategy, value):
        return self.execute('POST',
                            '/elements',
                            'findElements',
                            self._get_locator(strategy, value))

    def _get_locator(self, strategy, value):
        if self.mode == 'strict':
            return {'strategy': strategy, 'value': value}
        elif self.mode == 'compatibility':
            return {'using': strategy, 'value': value}
