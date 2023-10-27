FROM asciidoctor/docker-asciidoctor

# This custom docker image is only needed because the font config is broken
# in the upstream image which makes unnecessary noise when a plant uml diagram
# in the documentation is generated.
RUN apk add --no-cache fontconfig && \
    rm -rf /var/cache/fontconfig && \
    fc-cache -rs

CMD ["/bin/bash"]
