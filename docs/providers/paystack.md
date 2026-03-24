# Paystack Integration Guide

## Account Setup and API Key Generation

1. **Sign Up / Login**: Navigate to the [Paystack Dashboard](https://dashboard.paystack.com/).
2. **Test Mode**: Ensure the top bar is toggled to **Test Mode**.
3. **API Keys**: Navigate to **Settings > API Keys & Webhooks**. You will need:
   - Secret Key
   - Public Key
   - (Generate or define your own Webhook Secret / Signature check logically matching their payload model).

## Required Environment Variables

Add the following to your `.env` file or mapping properties:

```env
# Required for authorizing calls to paystack
PAYSTACK_SECRET_KEY=sk_test_xxxxxx
PAYSTACK_PUBLIC_KEY=pk_test_xxxxxx
# Used by the WebhookProcessor to perform HMAC SHA512 validation
PAYSTACK_WEBHOOK_SECRET=sk_test_xxxxxx # (Often the secret key itself in Paystack test mode)
# Config variables
PAYSTACK_BASE_URL=https://api.paystack.co
PAYSTACK_TIMEOUT_SECS=30
PAYSTACK_MAX_RETRIES=3
```

## Webhook Endpoint Configuration

In **Settings > API Keys & Webhooks**:
- Set your Test Webhook URL to: `https://api.yourdomain.com/webhooks/paystack`
- Note: Paystack allows selecting which events to listen to. Subscribe to:
  - `charge.success`
  - `transfer.success`
  - `transfer.failed`

## Webhook Signature Verification

Paystack generates a signature using **HMAC SHA512** of the payload with the Secret Key, passing it in the `x-paystack-signature` header.
Our platform reads this header, computes `HMAC_SHA512(request_body, PAYSTACK_WEBHOOK_SECRET)`, and verifies equality. If invalid, it returns `WebhookProcessorError::InvalidSignature`.

## Sandbox Testing Procedures

### Simulating Payments (Success)
Use Paystack's test cards (e.g., standard `4084084084084081` with any future expiry/CVV). Enter the testing OTP `123456` if prompted.

### Simulating Payments (Failure)
Use the Paystack failure test card (e.g., `4084084084084082`).

### Simulating Withdrawals (Transfers)
When initiating a B2C transfer to a recipient, Paystack handles sandbox transfers by allowing you to simulate the outcome. Inside the Paystack transfer dashboard, test mode allows triggering success via the Transfers menu, generating `transfer.success`. 

## Known Sandbox Limitations and Workarounds

- **Test Transfers**: Test transfers don't automatically complete without triggering them via Dashboard or a simulated mock response in specific environments.
- **Simulated OTPs**: Ensure test environments are allowed to skip real OTP checks, as strict validation might fail if bridging test keys in a live widget configuration.

## Common Error Codes and Resolution Steps

- **Signature Mismatch (`InvalidSignature`)**: The `PAYSTACK_WEBHOOK_SECRET` might not be identical to the key used for making the body. Paystack typically uses the `SECRET_KEY` as the webhook secret.
- **IP Allowlisting**: Paystack webhooks come from specific IP ranges. If testing remotely and not receiving webhooks, ensure VPC firewalls aren't blocking Paystack's IPs (52.31.139.75, 52.49.173.169, 52.214.14.220).
- **Duplicate Reference**: Paystack immediately rejects identical string references. The backend must enforce UUID generation for each unique transaction.
