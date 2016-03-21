import webdriver
session = webdriver.Session('127.0.0.1', 7000)
session.start()

session.url = 'http://google.com'
print session.url
