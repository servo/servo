cd build
../configure
make tidy && make -j2 && make check-servo && make check-content && make check-ref-cpu
WPTARGS="--processes=4" make check-wpt
