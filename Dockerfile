FROM gcr.io/distroless/cc-debian12:nonroot

WORKDIR /chokotto

COPY --chown=nonroot:nonroot target/release/chokotto2 /chokotto/chokotto2

USER nonroot:nonroot

ENTRYPOINT ["/chokotto/chokotto2"]

