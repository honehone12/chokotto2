FROM ubuntu:24.04

WORKDIR /chokotto

ARG UID=1000
ARG GID=1000

RUN groupadd -g $GID choko && useradd -m -u $UID -g choko choko

USER choko

COPY --chown=$UID:$GID target/release/chokotto2 /chokotto/chokotto2

ENTRYPOINT ["/chokotto/chokotto2"]
