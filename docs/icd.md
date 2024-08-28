# Interface Control Document (ICD) - `svc-contact`


![Aetheric Banner](https://github.com/aetheric-oss/.github/raw/main/assets/doc-banner.png)


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
[Requirements - `svc-contact`](https://nocodb.aetheric.nl/dashboard/#/nc/view/a2df942d-fcd7-47c0-9d8b-83b7df5698d1) | Requirements and user stories for this microservice.
[Concept of Operations - `svc-contact`](./conops.md) | Defines the motivation and duties of this microservice.
[Software Design Document (SDD) - `svc-contact`](./sdd.md) | Specifies the internal activity of this microservice.

## Frameworks

See the High-Level ICD.

## :speech_balloon: REST

### Files

Filename | Description
--- | ---
`openapi/types.rs` | Data types used for REST requests and replies.
`client-rest/src/lib.rs` | Imports the REST types file to create the `svc-cargo-client-rest` library, usable by other Rust crates.

### Authentication

See the High-Level Services ICD.

### Endpoints

See our [public documentation](https://www.arrowair.com/docs/documentation/services/api/rest/develop#tag/svc-contact) for a full API.

| HTTP Method | Description |
| --- | --- |
| POST | Given an email and display name, create a user record in svc-storage. See the SignupRequest body.

## gRPC

### Files

These interfaces are defined in a protocol buffer file, `proto/grpc.proto`.

### Integrated Authentication & Encryption

See the High-Level ICD.

### gRPC Server Methods ("Services")

| Service | Description |
| ---- | ---- |
| `cargoConfirmation` | Inform svc-contact to issue an email or text to a customer, informing them that an itinerary has been created.

### gRPC Client Messages ("Requests")

| Request | Description |
| ------    | ------- |
| `CargoConfirmationRequest` | Contains a parcel ID and itinerary ID for svc-contact, which is sufficient to obtain all of the other necessary information from svc-storage.
