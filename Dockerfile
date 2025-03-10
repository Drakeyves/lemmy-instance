FROM dessalines/lemmy:0.19.3

COPY config/config.hjson /config/config.hjson

EXPOSE 8536

CMD ["lemmy_server"] 