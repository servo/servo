# -*- coding: utf-8 -*-
for i in range(1000):
    exec("def test_func_%d(): pass" % i)
