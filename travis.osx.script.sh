cd build
../configure
make tidy && make -j2 && make check-servo && make check-content && make check-ref-cpu
