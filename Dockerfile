FROM gcr.io/distroless/cc-debian12:nonroot

WORKDIR /chokotto

COPY target/release/chokotto2 /chokotto/chokotto2

USER nonroot:nonroot

ENTRYPOINT ["/chokotto/chokotto2"]

