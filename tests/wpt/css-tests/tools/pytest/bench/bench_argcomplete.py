

# 10000 iterations, just for relative comparison
#                      2.7.5     3.3.2
# FilesCompleter       75.1109   69.2116
# FastFilesCompleter    0.7383    1.0760


if __name__ == '__main__':
    import sys
    import timeit
    from argcomplete.completers import FilesCompleter
    from _pytest._argcomplete import FastFilesCompleter
    count = 1000 # only a few seconds
    setup = 'from __main__ import FastFilesCompleter\nfc = FastFilesCompleter()'
    run = 'fc("/d")'
    sys.stdout.write('%s\n' % (timeit.timeit(run,
                                setup=setup.replace('Fast', ''), number=count)))
    sys.stdout.write('%s\n' % (timeit.timeit(run, setup=setup, number=count)))
