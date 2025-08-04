FROM rust:1.88-slim-bullseye
WORKDIR /app

LABEL maintainer="gmedia.id"

ARG NODE_VERSION=22
ARG MYSQL_CLIENT="mariadb-client"

ENV TZ=Asia/Jakarta

RUN ln -snf /usr/share/zoneinfo/$TZ /etc/localtime \
    && echo $TZ > /etc/timezone \
    && echo "Acquire::http::Pipeline-Depth 0;" > /etc/apt/apt.conf.d/99custom \
    && echo "Acquire::http::No-Cache true;" >> /etc/apt/apt.conf.d/99custom \
    && echo "Acquire::BrokenProxy    true;" >> /etc/apt/apt.conf.d/99custom \
    && apt-get update \
    && apt-get install -y --no-install-recommends \
        $MYSQL_CLIENT \
        build-essential \
        ca-certificates \
        cron \
        curl \
        dnsutils \
        fswatch \
        git \
        gnupg \
        gosu \
        iputils-ping \
        libcap2-bin \
        libmariadb-dev \
        libssl-dev \
        nano \
        pkg-config \
        redis-tools \
        supervisor \
        telnet \
        zip \    
    && cargo install --locked \
        sea-orm-cli \
        watchexec-cli \
        cargo-udeps \
        cargo-llvm-cov \
        cargo-tarpaulin \
    && cargo install --locked cargo-audit --features fix \
    && rustup component add rustfmt --toolchain nightly \
    && rustup component add llvm-tools-preview \
    && curl -L --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-from-binstall-release.sh | bash \
    && cargo binstall cargo-nextest --secure -y \
    && mkdir -p /etc/apt/keyrings \
    && curl -fsSL https://deb.nodesource.com/gpgkey/nodesource-repo.gpg.key | gpg --dearmor -o /etc/apt/keyrings/nodesource.gpg \
    && echo "deb [signed-by=/etc/apt/keyrings/nodesource.gpg] https://deb.nodesource.com/node_$NODE_VERSION.x nodistro main" > /etc/apt/sources.list.d/nodesource.list \
    && apt-get update \
    && apt-get install -y --no-install-recommends \
        nodejs \
    && apt-get -y autoremove \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/* /tmp/* /var/tmp/* \
    && npm install --ignore-scripts -g \
        npm

# Copy supervisord configuration file
COPY ./docker/1.88-dev/supervisord.conf /etc/supervisor/conf.d/supervisord.conf

# Expose the port the application will run on
EXPOSE 8080

# Run supervisord
CMD ["/usr/bin/supervisord", "-c", "/etc/supervisor/conf.d/supervisord.conf"]
