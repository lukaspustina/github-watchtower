# -*- mode: dockerfile -*-
#
# An example Dockerfile showing how to add new static C libraries using
# musl-gcc.

FROM ekidd/rust-musl-builder:stable

# Build a static copy of zlib.
#
# EXAMPLE ONLY! libz is already included.
RUN sudo apt-get update
RUN sudo apt-get install -y m4 clang

RUN VERS=6.1.2 && \
    cd /home/rust/libs && \
    curl -LO https://gmplib.org/download/gmp/gmp-$VERS.tar.bz2 && \
    tar xjf gmp-$VERS.tar.bz2 && cd gmp-$VERS && \
    CC=musl-gcc ./configure --enable-static --disable-shared --prefix=/usr/local/musl && \
    make && sudo make install && \
    cd .. && rm -rf gmp-$VERS.tar.gz gmp-$VERS

RUN VERS=3.4.1 && \
    cd /home/rust/libs && \
    curl -LO https://ftp.gnu.org/gnu/nettle/nettle-$VERS.tar.gz && \
    tar xzf nettle-$VERS.tar.gz && cd nettle-$VERS && \
    CFLAGS=-I/usr/local/musl/include LDFLAGS=-L/usr/local/musl/lib CC=musl-gcc ./configure --enable-static --disable-shared --prefix=/usr/local/musl && \
    make && sudo make install && \

    cd .. && rm -rf nettle-$VERS.tar.gz nettle-$VERS

RUN sudo apt-get clean && sudo rm -rf /var/lib/apt/lists/*

ENV NETTLE_STATIC=yes \
    PKG_CONFIG_PATH=$PKG_CONFIG_PATH:/usr/local/musl/lib/pkgconfig
