FROM php:8.1-cli-alpine

RUN apk update && \
    # Keeping these.
    apk add --no-cache coreutils icu libzip memcached unzip zip && \
    # Discarding these after after installing the php extensions.
    apk add --no-cache icu-dev libzip-dev $PHPIZE_DEPS && \
    pecl install xdebug && \
    docker-php-ext-install bcmath intl sockets zip && \
    docker-php-ext-enable xdebug && \
    apk del --purge icu-dev libzip-dev $PHPIZE_DEPS

COPY --from=spiralscout/roadrunner:2.6.3 /usr/bin/rr /usr/bin/rr
COPY --from=composer:latest /usr/bin/composer /usr/bin/composer

WORKDIR /app

EXPOSE 44356
EXPOSE 44357

COPY php.ini /usr/local/etc/php/conf.d/php.ini

ENTRYPOINT ["rr", "--debug", "--WorkDir", "/app", "--config", "/app/.rr.dev.yaml", "serve"]