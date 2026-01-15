#!/bin/bash
set -eux

#
# Install Capnproto from source
#

CAPNP_VERSION="1.3.0" # update from `capnp-version`` in repo root
wget https://capnproto.org/capnproto-c++-${CAPNP_VERSION}.tar.gz
tar xzf capnproto-c++-${CAPNP_VERSION}.tar.gz
cd capnproto-c++-${CAPNP_VERSION}
./configure
make -j$(nproc)
make install
ldconfig
