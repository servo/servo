import webdriver
session = webdriver.Session('127.0.0.1', 7000)
session.start()
print(session.url)
session.url = 'http://www.google.com'

print(session.url)
