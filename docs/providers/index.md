# Payment Provider Architecture

## Overview

The platform uses a unified payment provider architecture to integrate multiple external payment gateways like Flutterwave, Paystack, and M-Pesa. This architecture ensures the platform handles all payment operations seamlessly, irrespective of the underlying gateway.

## Architecture

The system is built on an abstraction layer (often structured via the Factory pattern) that handles routing of payment payloads to the respective provider modules:

1. **Provider Factory**: The orchestrator evaluates transactions and routes API calls to the corresponding provider (e.g., Paystack or Flutterwave) using configured environment variables and endpoint routing.
2. **Standardized Interfaces**: Each provider implements a standardized rust trait or interface, guaranteeing consistency when creating payments, verifying statuses, and handling withdrawals.
3. **Webhook Processor**: A core `WebhookProcessor` accepts incoming `POST /webhooks/:provider` events. It automatically evaluates the configured provider string (e.g., `paystack`), finds the relevant secret key/hash config, validates the signature, and prevents replay attacks or double-processing.

## Webhook Routing 

All webhooks are pointing to:

```http
POST /webhooks/:provider
```

For instance:
- `POST /webhooks/paystack`
- `POST /webhooks/flutterwave`

The platform uses signature validation specific to each provider.

## Directory Structure

In this `providers` directory, you will find comprehensive guides for each supported provider:

- [Flutterwave Integration Guide](./flutterwave.md)
- [Paystack Integration Guide](./paystack.md)
- [M-Pesa Integration Guide](./mpesa.md)
- [Webhook Testing Guide](./webhook-testing.md)
- [Provider Feature Comparison](./feature-comparison.md)
