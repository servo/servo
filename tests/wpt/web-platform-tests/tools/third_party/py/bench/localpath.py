import py

class Listdir:
    numiter = 100000
    numentries = 100

    def setup(self):
        tmpdir = py.path.local.make_numbered_dir(self.__class__.__name__)
        for i in range(self.numentries):
            tmpdir.join(str(i))
        self.tmpdir = tmpdir

    def run(self):
        return self.tmpdir.listdir()

class Listdir_arg(Listdir):
    numiter = 100000
    numentries = 100

    def run(self):
        return self.tmpdir.listdir("47")

class Join_onearg(Listdir):
    def run(self):
        self.tmpdir.join("17")
        self.tmpdir.join("18")
        self.tmpdir.join("19")

class Join_multi(Listdir):
    def run(self):
        self.tmpdir.join("a", "b")
        self.tmpdir.join("a", "b", "c")
        self.tmpdir.join("a", "b", "c", "d")

class Check(Listdir):
    def run(self):
        self.tmpdir.check()
        self.tmpdir.check()
        self.tmpdir.check()

class CheckDir(Listdir):
    def run(self):
        self.tmpdir.check(dir=1)
        self.tmpdir.check(dir=1)
        assert not self.tmpdir.check(dir=0)

class CheckDir2(Listdir):
    def run(self):
        self.tmpdir.stat().isdir()
        self.tmpdir.stat().isdir()
        assert self.tmpdir.stat().isdir()

class CheckFile(Listdir):
    def run(self):
        self.tmpdir.check(file=1)
        assert not self.tmpdir.check(file=1)
        assert self.tmpdir.check(file=0)

if __name__ == "__main__":
    import time
    for cls in [Listdir, Listdir_arg,
                Join_onearg, Join_multi,
               Check, CheckDir, CheckDir2, CheckFile,]:

        inst = cls()
        inst.setup()
        now = time.time()
        for i in xrange(cls.numiter):
            inst.run()
        elapsed = time.time() - now
        print("%s: %d loops took %.2f seconds, per call %.6f" %(
               cls.__name__,
                cls.numiter, elapsed, elapsed / cls.numiter))
