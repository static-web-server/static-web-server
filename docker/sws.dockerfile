FROM scratch

LABEL maintainer=https://git.io/joseluisq

ADD bin /
ADD public /public

ENTRYPOINT ["/static-web-server"]
