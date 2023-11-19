#!/bin/bash

SLEEP=${1:-'10'}
EXIT=${2:-'0'}

echo "Sleeping for $SLEEP seconds...";

sleep $SLEEP;

echo "Waking up!";

exit $EXIT;