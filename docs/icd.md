# Interface Control Document (ICD) - `svc-contact`

<center>

<img src="https://github.com/aetheric-oss/.github/blob/main/assets/doc-banner.png" style="height:250px" />

</center>

## Overview

This document defines the gRPC and REST interfaces unique to the `svc-contact` microservice.

Attribute | Description
--- | ---
Status | Draft

## Related Documents

Document | Description
--- | ---
[High-Level Concept of Operations (CONOPS)](https://github.com/aetheric-oss/se-services/blob/develop/docs/conops.md) | Overview of Aetheric microservices.
[High-Level Interface Control Document (ICD)](https://github.com/aetheric-oss/se-services/blob/develop/docs/icd.md)  | Interfaces and frameworks common to all Aetheric microservices.
[Requirements - `svc-contact`](https://nocodb.arrowair.com/dashboard/#/nc/view/a2df942d-fcd7-47c0-9d8b-83b7df5698d1) | Requirements and user stories for this microservice.
[Concept of Operations - `svc-contact`](./conops.md) | Defines the motivation and duties of this microservice.
[Software Design Document (SDD) - `svc-contact`](./sdd.md) | Specifies the internal activity of this microservice.

## Frameworks

See the High-Level ICD.

## REST

This microservice implements no additional REST endpoints beyond the common REST interfaces (see High-Level ICD).

## gRPC

### Files

These interfaces are defined in a protocol buffer file, `proto/grpc.proto`.

### Integrated Authentication & Encryption

See the High-Level ICD.

### gRPC Server Methods ("Services")

| Service | Description |
| ---- | ---- |
| `GetExample` | This is an example Service.<br>Replace

### gRPC Client Messages ("Requests")

| Request | Description |
| ------    | ------- |
| `ExampleQuery` | A message to illustrate an example
