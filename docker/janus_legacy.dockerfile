FROM alpine:3.18

RUN apk add --no-cache \
    build-base \
    git \
    automake \
    autoconf \
    libtool \
    pkgconfig \
    glib-dev \
    jansson-dev \
    libconfig-dev \
    libmicrohttpd-dev \
    libwebsockets-dev \
    openssl-dev \
    libsrtp-dev \
    sofia-sip-dev \
    opus-dev \
    libogg-dev \
    curl-dev \
    lua5.3-dev \
    cmake \
    gtk-doc \
    python3 \
    py3-pip \
    meson \
    ninja \
    gnutls-dev \
    gobject-introspection-dev \
    gengetopt

# Build libnice from source
RUN cd /tmp \
    && git clone https://gitlab.freedesktop.org/libnice/libnice.git \
    && cd libnice \
    && meson setup --prefix=/usr build \
    && ninja -C build \
    && ninja -C build install

# Build janus-gateway from source
RUN cd /opt \
    && git clone --branch v0.15.1 https://github.com/meetecho/janus-gateway.git \
    && cd janus-gateway \
    && ./autogen.sh \
    && ./configure --prefix=/opt/janus \
        --disable-rabbitmq \
        --disable-mqtt \
        --disable-unix-sockets \
        --disable-data-channels \
    && make \
    && make install \
    && make configs

EXPOSE 8088 8089 8889 8000
EXPOSE 10000-10200/udp

WORKDIR /opt/janus

CMD ["/opt/janus/bin/janus"]
