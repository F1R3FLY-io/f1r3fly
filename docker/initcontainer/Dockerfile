FROM ghcr.io/graalvm/jdk:ol8-java17-22.3.3 as prebuilt

RUN sh -c '(echo "#!/usr/bin/env sh" && \
  curl -L https://github.com/com-lihaoyi/Ammonite/releases/download/2.5.11/2.13-2.5.11) > /usr/local/bin/amm && \
  chmod +x /usr/local/bin/amm'

RUN adduser non_root
USER non_root

ADD ./*.sc .

RUN amm -c 'val mods = os.list(os.pwd).filter(_.ext == "sc"); mods.foreach(interp.load.module)'

FROM ghcr.io/graalvm/jdk:ol8-java17-22.3.3

RUN microdnf update && microdnf upgrade -y && microdnf install ncurses

COPY --from=prebuilt /usr/local/bin/amm /usr/local/bin

RUN adduser non_root
ENV HOME /home/non_root

COPY --from=prebuilt /home/non_root/.cache $HOME/.cache
COPY --from=prebuilt /home/non_root/.ammonite $HOME/.ammonite
ADD ./*.sc .
RUN chown -R non_root .
RUN chown -R non_root $HOME

USER non_root
