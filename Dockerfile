FROM ubuntu:bionic AS builder
LABEL maintainer="Jacek Całusiński <forger@forger.pl>"

# Set locale (fix the locale warnings)
ENV LC_ALL="en_US.UTF-8" \
	LC_CTYPE="en_US.UTF-8" \
	LANG="en_US.UTF-8"

ENV DEBIAN_FRONTEND noninteractive

RUN apt-get update \
    && apt-get dist-upgrade -y --no-install-recommends \
    && apt-get install -y locales curl moreutils build-essential \
        ca-certificates libssl1.1 libcurl4-openssl-dev  libssl-dev pkg-config \
	&& apt-get clean \
	&& rm -rf /var/lib/apt/lists

RUN localedef -v -c -i en_US -f UTF-8 en_US.UTF-8 || :

RUN adduser --shell /bin/bash --disabled-login --disabled-password --gecos "" rust

WORKDIR /home/rust/

USER rust

RUN curl https://sh.rustup.rs -sSf | sh -s -- -y

ENV PATH="/home/rust/.cargo/bin:$PATH"

COPY --chown=rust . webcrawl
WORKDIR /home/rust/webcrawl

RUN cargo test --all \
    && cargo build --release \
    && cp target/release/webcrawl /home/rust/webcrawl \
    && cargo clean

FROM ubuntu:bionic
LABEL maintainer="Jacek Całusiński <forger@forger.pl>"

ENV DEBIAN_FRONTEND noninteractive

RUN apt-get update \
    && apt-get dist-upgrade -y --no-install-recommends \
    && apt-get install -y ca-certificates libssl1.1 gosu \
    && apt-get clean \
	&& rm -rf /var/lib/apt/lists

RUN adduser --shell /bin/bash --disabled-login --disabled-password --gecos "" webcrawl

COPY --from=builder --chown=webcrawl /home/rust/webcrawl/webcrawl /usr/local/bin/webcrawl

ENTRYPOINT ["gosu", "webcrawl", "/usr/local/bin/webcrawl"]
CMD ["--help"]
