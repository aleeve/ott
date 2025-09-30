FROM ghcr.io/cloudnative-pg/postgresql:17

USER root
RUN apt-get update && \
  apt-get install -y --no-install-recommends \
  postgresql-contrib \
  postgresql-17-pgvector \
  postgresql-17-cron \
  postgresql-17-partman;\
  rm -fr /tmp/* ; \
  rm -rf /var/lib/apt/lists/*;

USER 26
