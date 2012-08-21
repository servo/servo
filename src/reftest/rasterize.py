import sys, os
from selenium import webdriver

input = sys.argv[1]
output = sys.argv[2]

input = os.path.abspath(input)
output = os.path.abspath(output)

input = "file://" + input

print input
print output

driver = webdriver.Firefox()

driver.set_window_size(800, 700)
driver.get(input)

screenshot = driver.get_screenshot_as_file(output)

driver.quit()
