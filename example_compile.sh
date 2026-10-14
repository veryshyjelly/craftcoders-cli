#!/bin/sh
# Compile script for Clojure Redis implementation
# Run this to build your project before testing

set -e

echo "Building Clojure project..."
clj -T:build

# Copy the built JAR to the project directory for easy access
cp /tmp/codecrafters-build-redis-clojure/target.jar target.jar

echo "Build complete! JAR ready at: target.jar"