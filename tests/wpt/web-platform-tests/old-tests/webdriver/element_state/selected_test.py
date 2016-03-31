import os
import sys
import unittest

sys.path.insert(1, os.path.abspath(os.path.join(__file__, "../..")))
import base_test


class GetElementSelectedTest(base_test.WebDriverBaseTest):
    def test_selected_1(self):
        self.driver.get(self.webserver.where_is("element_state/res/element-selected.html"))
        element = self.driver.find_element_by_id("selected-1")

        self.assertEquals(element.is_selected(), True)

    def test_selected_2(self):
        self.driver.get(self.webserver.where_is("element_state/res/element-selected.html"))
        element = self.driver.find_element_by_id("selected-2")

        self.assertEquals(element.is_selected(), True)

    def test_selected_3(self):
        self.driver.get(self.webserver.where_is("element_state/res/element-selected.html"))
        element = self.driver.find_element_by_id("selected-3")

        self.assertEquals(element.is_selected(), True)

    def test_selected_4(self):
        self.driver.get(self.webserver.where_is("element_state/res/element-selected.html"))
        element = self.driver.find_element_by_id("selected-4")

        self.assertEquals(element.is_selected(), True)

    def test_selected_5(self):
        self.driver.get(self.webserver.where_is("element_state/res/element-selected.html"))
        element = self.driver.find_element_by_id("selected-5")

        self.assertEquals(element.is_selected(), True)

    def test_selected_6(self):
        self.driver.get(self.webserver.where_is("element_state/res/element-selected.html"))
        element = self.driver.find_element_by_id("selected-6")

        self.assertEquals(element.is_selected(), True)

    def test_selected_7(self):
        self.driver.get(self.webserver.where_is("element_state/res/element-selected.html"))
        element = self.driver.find_element_by_id("selected-7")

        self.assertEquals(element.is_selected(), True)

    def test_selected_8(self):
        self.driver.get(self.webserver.where_is("element_state/res/element-selected.html"))
        element = self.driver.find_element_by_id("selected-8")

        self.assertEquals(element.is_selected(), True)

    def test_selected_9(self):
        self.driver.get(self.webserver.where_is("element_state/res/element-selected.html"))
        element = self.driver.find_element_by_id("selected-9")

        self.assertEquals(element.is_selected(), True)

    def test_selected_10(self):
        self.driver.get(self.webserver.where_is("element_state/res/element-selected.html"))
        element = self.driver.find_element_by_id("selected-10")

        self.assertEquals(element.is_selected(), True)

    def test_selected_11(self):
        self.driver.get(self.webserver.where_is("element_state/res/element-selected.html"))
        element = self.driver.find_element_by_id("selected-11")

        self.assertEquals(element.is_selected(), True)

    def test_selected_12(self):
        self.driver.get(self.webserver.where_is("element_state/res/element-selected.html"))
        element = self.driver.find_element_by_id("selected-12")

        self.assertEquals(element.is_selected(), True)

    def test_selected_13(self):
        self.driver.get(self.webserver.where_is("element_state/res/element-selected.html"))
        element = self.driver.find_element_by_id("selected-13")

        self.assertEquals(element.is_selected(), True)

    def test_selected_14(self):
        self.driver.get(self.webserver.where_is("element_state/res/element-selected.html"))
        element = self.driver.find_element_by_id("selected-14")

        self.assertEquals(element.is_selected(), True)

    def test_selected_15(self):
        self.driver.get(self.webserver.where_is("element_state/res/element-selected.html"))
        element = self.driver.find_element_by_id("selected-15")

        self.assertEquals(element.is_selected(), True)

    def test_unselected_1(self):
        self.driver.get(self.webserver.where_is("element_state/res/element-selected.html"))
        element = self.driver.find_element_by_id("unselected-1")

        self.assertEquals(element.is_selected(), False)

    def test_unselected_2(self):
        self.driver.get(self.webserver.where_is("element_state/res/element-selected.html"))
        element = self.driver.find_element_by_id("unselected-2")

        self.assertEquals(element.is_selected(), False)

    def test_unselected_3(self):
        self.driver.get(self.webserver.where_is("element_state/res/element-selected.html"))
        element = self.driver.find_element_by_id("unselected-3")

        self.assertEquals(element.is_selected(), False)

    def test_unselected_4(self):
        self.driver.get(self.webserver.where_is("element_state/res/element-selected.html"))
        element = self.driver.find_element_by_id("unselected-4")

        self.assertEquals(element.is_selected(), False)

    def test_unselected_5(self):
        self.driver.get(self.webserver.where_is("element_state/res/element-selected.html"))
        element = self.driver.find_element_by_id("unselected-5")

        self.assertEquals(element.is_selected(), False)

    def test_unselected_6(self):
        self.driver.get(self.webserver.where_is("element_state/res/element-selected.html"))
        element = self.driver.find_element_by_id("unselected-6")

        self.assertEquals(element.is_selected(), False)

    def test_unselected_7(self):
        self.driver.get(self.webserver.where_is("element_state/res/element-selected.html"))
        element = self.driver.find_element_by_id("unselected-7")

        self.assertEquals(element.is_selected(), False)

    def test_unselected_8(self):
        self.driver.get(self.webserver.where_is("element_state/res/element-selected.html"))
        element = self.driver.find_element_by_id("unselected-8")

        self.assertEquals(element.is_selected(), False)

    def test_unselected_9(self):
        self.driver.get(self.webserver.where_is("element_state/res/element-selected.html"))
        element = self.driver.find_element_by_id("unselected-9")

        self.assertEquals(element.is_selected(), False)

    def test_unselected_10(self):
        self.driver.get(self.webserver.where_is("element_state/res/element-selected.html"))
        element = self.driver.find_element_by_id("unselected-10")

        self.assertEquals(element.is_selected(), False)

    def test_unselected_11(self):
        self.driver.get(self.webserver.where_is("element_state/res/element-selected.html"))
        element = self.driver.find_element_by_id("unselected-11")

        self.assertEquals(element.is_selected(), False)

    def test_unselected_12(self):
        self.driver.get(self.webserver.where_is("element_state/res/element-selected.html"))
        element = self.driver.find_element_by_id("unselected-12")

        self.assertEquals(element.is_selected(), False)

    def test_unselected_13(self):
        self.driver.get(self.webserver.where_is("element_state/res/element-selected.html"))
        element = self.driver.find_element_by_id("unselected-13")

        self.assertEquals(element.is_selected(), False)

    def test_unselected_14(self):
        self.driver.get(self.webserver.where_is("element_state/res/element-selected.html"))
        element = self.driver.find_element_by_id("unselected-14")

        self.assertEquals(element.is_selected(), False)

    def test_unselected_15(self):
        self.driver.get(self.webserver.where_is("element_state/res/element-selected.html"))
        element = self.driver.find_element_by_id("unselected-15")

        self.assertEquals(element.is_selected(), False)

    def test_unselected_16(self):
        self.driver.get(self.webserver.where_is("element_state/res/element-selected.html"))
        element = self.driver.find_element_by_id("unselected-16")

        self.assertEquals(element.is_selected(), False)

    def test_unselected_17(self):
        self.driver.get(self.webserver.where_is("element_state/res/element-selected.html"))
        element = self.driver.find_element_by_id("unselected-17")

        self.assertEquals(element.is_selected(), False)

    def test_unselected_18(self):
        self.driver.get(self.webserver.where_is("element_state/res/element-selected.html"))
        element = self.driver.find_element_by_id("unselected-18")

        self.assertEquals(element.is_selected(), False)


if __name__ == "__main__":
    unittest.main()
