import webdriver
session = webdriver.Session('127.0.0.1', 7000)
session.start()
<<<<<<< HEAD
print(session.url)
session.url = 'http://www.google.com'

print(session.url)
=======

session.url = 'http://google.com'
print session.url
>>>>>>> 0153e5b052f126dc78ce947f74ad0b9fad6f465c
