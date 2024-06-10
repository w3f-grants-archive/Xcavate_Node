FROM paritytech/ci-linux:production as build

WORKDIR /Xcavate_Node
COPY . .
RUN cargo build --release

FROM ubuntu:20.04
WORKDIR /node


COPY --from=build /Xcavate_Node/target/release/node-template .


RUN apt update && \
    apt install -y ca-certificates && \
    update-ca-certificates && \
    apt remove ca-certificates -y && \
    rm -rf /var/lib/apt/lists/*

EXPOSE 9944

CMD [ "./node-template", "--dev", "--ws-external", "--rpc-methods=Unsafe" ]