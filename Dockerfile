FROM docker.io/paritytech/ci-unified:latest as builder

WORKDIR /sanctuary
COPY . /sanctuary

RUN cargo fetch
RUN cargo build --locked --release

FROM docker.io/parity/base-bin:latest

COPY --from=builder /sanctuary/target/release/sanctuary-node /usr/local/bin

USER root
RUN useradd -m -u 1001 -U -s /bin/sh -d /sanctuary sanctuary && \
	mkdir -p /data /sanctuary/.local/share && \
	chown -R sanctuary:sanctuary /data && \
	ln -s /data /sanctuary/.local/share/sanctuary && \
# unclutter and minimize the attack surface
	rm -rf /usr/bin /usr/sbin && \
# check if executable works in this container
	/usr/local/bin/sanctuary-node --version

USER sanctuary

EXPOSE 30333 9933 9944 9615
VOLUME ["/data"]

ENTRYPOINT ["/usr/local/bin/sanctuary-node"]
