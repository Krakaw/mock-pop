#!/bin/bash

(sleep 1
echo 'USER a'
sleep 1
echo 'PASS b'
sleep 1
echo "LIST"
sleep 1
echo "STAT"
sleep 1
echo "QUIT") | nc localhost 1110


(sleep 1
echo 'USER a
PASS b
LIST
STAT
QUIT'
) | nc localhost 1110
