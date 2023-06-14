ARG CROSS_BASE_IMAGE
FROM $CROSS_BASE_IMAGE

RUN rm -rf /usr/local/go
RUN curl -L https://go.dev/dl/go1.20.5.linux-amd64.tar.gz | tar -xz -C /usr/local

ENV PATH /usr/local/go/bin:$PATH
# Verify that `go` is in the path
RUN go version
