#!/bin/bash
docker build -t stor.highloadcup.ru/rally/centipede_runner:latest -f "$1" .
docker push stor.highloadcup.ru/rally/centipede_runner:latest