FROM scratch

LABEL maintainer=https://github.com/joseluisq

ADD bin /
ADD public /public

ENTRYPOINT ["/rust-web-server"]
