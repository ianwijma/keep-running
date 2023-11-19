#!/bin/bash

SLEEP=${1:-'10'}

echo "Sleeping for $SLEEP seconds...";

sleep $SLEEP;

echo "Waking up!";

exit 1;