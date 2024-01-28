ARG DEBIAN_VER=bookworm
FROM golang:1-${DEBIAN_VER}

RUN apt-get update
RUN apt-get -y install devscripts

WORKDIR /root
RUN mkdir /root/OUTPUT

COPY go.mod .
COPY go.sum .
RUN go mod download

COPY cmd cmd
COPY *.go ./
RUN CGO_ENABLED=0 go build

COPY LICENSE README.md ./
COPY debian debian
