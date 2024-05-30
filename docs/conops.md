# Concept of Operations - `svc-contact`


![Aetheric Banner](https://github.com/aetheric-oss/.github/raw/main/assets/doc-banner.png)


Attribute | Description
--- | ---
Maintainer | [@aetheric-oss/dev-realm](https://github.com/orgs/aetheric-oss/teams)
Status | Draft

## Overview

This service takes requests from other microservices to send transactional information to a user (e.g. flight confirmations). It connects to external email and SMS services.

## Related Documents

Document | Description
--- | ---
[High-Level Concept of Operations (CONOPS)](https://github.com/aetheric-oss/se-services/blob/develop/docs/conops.md) | Overview of Aetheric microservices.
[High-Level Interface Control Document (ICD)](https://github.com/aetheric-oss/se-services/blob/develop/docs/icd.md)  | Interfaces and frameworks common to all Aetheric microservices.
[Requirements - `svc-contact`](https://nocodb.aetheric.nl/dashboard/#/nc/view/a2df942d-fcd7-47c0-9d8b-83b7df5698d1) | Requirements and user stories for this microservice.
[Interface Control Document (ICD) - `svc-contact`](./icd.md) | Defines the inputs and outputs of this microservice.
[Software Design Document (SDD) - `svc-contact`](./sdd.md) | Specifies the internal activity of this microservice.

## Motivation

This module has connections to external services (like SendGrid, Mailchimp, or Postmark).

Users will receive updates via this service and an external email/sms delivery service.

## Needs, Goals and Objectives of Envisioned System

## Overview of System and Key Elements

## External Interfaces
See the ICD for this microservice.

## Proposed Capabilities

## Modes of Operation

## Operational Scenarios, Use Cases and/or Design Reference Missions

## Nominal & Off-Nominal Conditions

## Physical Environment

See the High-Level CONOPS.

## Support Environment

See the High-Level CONOPS.

## Impact Considerations

## Environmental Impacts

## Organizational Impacts

## Technical Impacts

## Risks and Potential Issues

## Appendix A: Citations

## Appendix B: Acronyms & Glossary
