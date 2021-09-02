FROM golang
RUN GO111MODULE=off go get -u github.com/rexray/gocsi/csc
ENTRYPOINT [ "/go/bin/csc" ]
