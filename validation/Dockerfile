FROM alpine:3.19

# rust + cargo to rebuild push_swap.rs with `make re`
# make for the Makefile each model produces
# gcompat to run the glibc-built checker_linux on musl alpine
RUN apk add --no-cache rust cargo make gcompat

WORKDIR /work
