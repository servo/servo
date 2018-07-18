import py

for i in range(1000):
    py.builtin.exec_("def test_func_%d(): pass" % i)
