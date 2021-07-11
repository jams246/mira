#!/bin/sh

WORKDIR="/app"
DEBUG="--debug"
CONFIG=".rr.dev.yaml"
ENV=$(echo $APP_ENV | tr "[:upper:]" "[:lower:]")
if [ $ENV = "prod" ]; then
	if [ ! -f "$WORKDIR/.rr.yaml" ]; then
		echo "$WORKDIR/.rr.yaml file not found."
		exit 1
	fi
	CONFIG=".rr.yaml"
	DEBUG=""
fi

rr "$DEBUG" --WorkDir "$WORKDIR" --config "$WORKDIR/$CONFIG" serve