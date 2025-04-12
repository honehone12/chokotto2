FROM ubuntu:latest

WORKDIR /

COPY target/release/chokotto2 /chokotto2

CMD ["/chokotto2"]

