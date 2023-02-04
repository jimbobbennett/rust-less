# rust-less

A serverless host written in Rust. A complete experiment for me to learn Rust! The aim is to create something that can be hosted on a Raspberry Pi.

This is not true serverless in that it won't scale across hardware, instead this will scale up and down instances of executors to respond to serverless events.

This will also only respond to web triggers.

The rough architecture:

* A core runtime that runs on a Raspberry Pi listening to web calls
* A framework for creating Rust crates with a defined API that expose the 'function' called by the serverless runtime
* A mechanism for defining what web hook will call the 'function'
* A way to pass data to the 'function' as JSON, and pass back results including an HTTP status code and an optional JSON body


## Notes on the structure

* Each 'function app' will be source code for Rust library that implements one or more structs with a certain trait, along with configuration that maps resources to the struct.
* The 'runtime' will be called to register each function app. It will upload the source code for the library, then compile it into a runner app. This host runner app will have the basic runner defined as code, and will compile that with the libraries.
* Each function app will be run in a docker container on the host platform
* There will be an orchestrator that runs the docker containers, and forwards web request to a port on each docker container, returning the result.
* The host will also run locally for debugging purposes

## CLI

* Need a CLI to run that will connect to the host
* CLI registers functions with the host
* Host needs a DB to store route to built docker container

## Tasks

* Docker container
    * Core binary host code
    * Ability to add crate with function
    * Exposure of function via web interface

* CLI
    * Detects main host
    * Provides registration commands

* Main host
    * Listens on web requests
    * Ability to register web hooks
    * Ability to register code with routes
    * Ability to build docker containers for routes
    * Ability to register docker container to multiple routes

