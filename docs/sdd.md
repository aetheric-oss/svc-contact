# Software Design Document (SDD) - `svc-contact` 


![Aetheric Banner](https://github.com/aetheric-oss/.github/raw/main/assets/doc-banner.png)

## Overview

This document details the software implementation of `svc-contact`.

This service takes requests from other microservices to send transactional information to a user (e.g. flight confirmations). It connects to external email and SMS services.

Attribute | Description
--- | ---
Status | Draft

## Related Documents

Document | Description
--- | ---
[High-Level Concept of Operations (CONOPS)](https://github.com/aetheric-oss/se-services/blob/develop/docs/conops.md) | Overview of Aetheric microservices.
[High-Level Interface Control Document (ICD)](https://github.com/aetheric-oss/se-services/blob/develop/docs/icd.md)  | Interfaces and frameworks common to all Aetheric microservices.
[Requirements - `svc-contact`](https://nocodb.aetheric.nl/dashboard/#/nc/view/a2df942d-fcd7-47c0-9d8b-83b7df5698d1) | Requirements and user stories for this microservice.
[Concept of Operations - `svc-contact`](./conops.md) | Defines the motivation and duties of this microservice.
[Interface Control Document (ICD) - `svc-contact`](./icd.md) | Defines the inputs and outputs of this microservice.

## :dna: Module Attributes

Attribute | Applies | Explanation
--- | --- | ---
Safety Critical | No | This is a client-facing process that is not essential to the function of the underlying services network.

## :gear: Logic 

### Initialization

At initialization this service creates two servers on separate threads: a GRPC server and a REST server.

The REST server expects the following environment variables to be set:
- `DOCKER_PORT_REST` (default: `8000`)

The GRPC server expects the following environment variables to be set:
- `DOCKER_PORT_GRPC` (default: `50051`)

### Control Loop

As a REST and GRPC server, this service awaits requests and executes handlers.

Some handlers **require** the following environment variables to be set:
- `STORAGE_HOST_GRPC`
- `STORAGE_PORT_GRPC`

This information allows `svc-contact` to connect to other microservices to obtain information requested by the client.

:exclamation: These environment variables will *not* default to anything if not found. In this case, requests involving the handler will result in a `503 SERVICE UNAVAILABLE`.

For detailed sequence diagrams regarding request handlers, see [REST Handlers](#speech_balloon-rest-handlers).

### Cleanup

None

## :speech_balloon: REST Handlers

### `signup` Handler

The client will request to "sign up" with the network. They will provide a form of credential.

This handler makes a request to `svc-storage`.
