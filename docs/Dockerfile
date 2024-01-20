FROM squidfunk/mkdocs-material

RUN set -eux \
    && apk upgrade --update-cache -a \
    && pip3 install \
        mkdocs-material \
        mkdocs-git-revision-date-localized-plugin \
    && true

COPY ./entrypoint.sh /

WORKDIR /docs

EXPOSE 8000

ENTRYPOINT ["/entrypoint.sh"]

CMD ["mkdocs", "serve", "--dev-addr=0.0.0.0:8000"]
