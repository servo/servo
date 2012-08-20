import sys
from selenium import webdriver

input = sys.argv[1]
output = sys.argv[2]

driver = webdriver.Firefox()

driver.set_window_size(800, 600)
driver.get(file)

screenshot = driver.get_screenshot_as_file(output)

driver.quit()
