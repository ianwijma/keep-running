#!/bin/bash

SLEEP=${1:-'10'}
EXIT=${2:-'0'}

echo "Sleeping for $SLEEP seconds...";

sleep $SLEEP;

echo "Waking up!";

if [ $EXIT > 0 ]; then
  >&2 echo "Exiting with error code $EXIT";
else
  echo "Exiting with error code $EXIT";
fi

exit $EXIT;