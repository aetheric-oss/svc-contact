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
[Requirements - `svc-contact`](https://nocodb.arrowair.com/dashboard/#/nc/view/a2df942d-fcd7-47c0-9d8b-83b7df5698d1) | Requirements and user stories for this microservice.
[Concept of Operations - `svc-contact`](./conops.md) | Defines the motivation and duties of this microservice.
[Interface Control Document (ICD) - `svc-contact`](./icd.md) | Defines the inputs and outputs of this microservice.

## Module Attributes

Attribute | Applies | Explanation
--- | --- | ---
Safety Critical | ? | 
Realtime | ? |

## Global Variables

**Statically Allocated Queues**

FIXME

## Logic

### Initialization

FIXME Description of activities at init

### Loop

FIXME Description of activities during loop

### Cleanup

FIXME Description of activities at cleanup

## Interface Handlers

FIXME - What internal activities are triggered by messages at this module's interfaces?

## Tests

FIXME

### Unit Tests

FIXME
