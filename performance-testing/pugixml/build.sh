#!/bin/bash

mkdir target
g++ -O3 main.cpp pugixml.cpp -o target/test-\pugixml

