FROM debian:buster-20200422

# Debian 10 doesn't have openjdk-8, so add the Debian 9 repository, which contains it.
RUN sed s/buster/stretch/ /etc/apt/sources.list | tee /etc/apt/sources.list.d/stretch.list

COPY setup.sh /root
RUN cd /root && ./setup.sh

RUN useradd -d /home/worker -s /bin/bash -m worker
USER worker
WORKDIR /home/worker
CMD /bin/bash
