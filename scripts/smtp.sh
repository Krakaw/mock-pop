#!/bin/bash

(sleep 1
echo 'helo a'
sleep 1
echo 'mail from:<b@a.com>'
sleep 1
echo "rcpt to:<a@a.com>"
sleep 1
echo "data
test"
sleep 1
echo "."
sleep 1
echo "quit") | telnet localhost 2525
