# Flutterwave Integration Guide

## Account Setup and API Key Generation

1. **Sign Up / Login**: Navigate to [Flutterwave's Dashboard](https://dashboard.flutterwave.com/).
2. **Switch to Test Mode**: For development, ensure you toggle "Test Mode" on.
3. **API Keys**: Go to **Settings > API Keys** to find your keys. You will need:
   - Secret Key
   - Public Key
   - Webhook Secret Hash (you define this in the dashboard).

## Required Environment Variables

Configure the following in your `.env` string or `config.rs`.

```env
# Required for making authenticated API calls
FLUTTERWAVE_SECRET_KEY=FLWSECK_TEST-xxxxxxx-X
# Required for Webhook Validation. Must match what is provided in the dashboard.
FLUTTERWAVE_WEBHOOK_SECRET=your_secret_hash_value_here
FLUTTERWAVE_WEBHOOK_HASH=your_secret_hash_value_here # Also checked as fallback
# Optional / Timeout Settings
FLUTTERWAVE_BASE_URL=https://api.flutterwave.com/v3
FLUTTERWAVE_TIMEOUT_SECS=30
PAYMENT_TIMEOUT_SECONDS=60
FLUTTERWAVE_MAX_RETRIES=3
```

## Webhook Endpoint Configuration

In the Flutterwave dashboard, go to **Settings > Webhooks**.
1. **Webhook URL**: Input the platform's public facing endpoint mapping to flutterwave.
   - Example: `https://api.yourdomain.com/webhooks/flutterwave`
2. **Secret Hash**: Enter the value configured in `FLUTTERWAVE_WEBHOOK_SECRET`. 

**Events to Subscribe to:**
- `charge.completed`
- `transfer.completed`
- `transfer.failed`

> The platform's `WebhookProcessor` parses `charge.completed` for pay-ins and `transfer.completed`/`transfer.failed` for pay-outs.

## Webhook Signature Verification

Flutterwave passes the secret hash via the custom header `verif-hash`. The platform validates this by comparing it against the `FLUTTERWAVE_WEBHOOK_SECRET`. 

If `verif-hash` doesn't match the configured environment variable, the system returns an `InvalidSignature` error to prevent injection.

## Sandbox Testing Procedures

### Simulating Successful Payments
Use Flutterwave test cards (e.g., standard testing Pan ending in `2004`) during checkout. This auto-approves.

### Simulating Failed Payments
Use a failure test card (e.g., Pan ending in `9995`). Do not use OTP, or input an incorrect OTP when prompted.

### Simulating Withdrawals
When executing a withdrawal via the `transfer` API in sandbox, providing specific standard bank accounts (like `044` Access Bank) acts differently. Flutterwave will trigger a `transfer.completed` asynchronously in test mode for most accounts to allow webhook debugging.

## Known Limitations and Workarounds

- **Webhook Delay in Sandbox**: In the test environment, `transfer.completed` webhooks can occasionally be delayed. Ensure you wait up to 5 minutes before assuming the webhook handler failed.
- **Card Currency Limits**: Sandbox limits some multi-currency cross-tests. Always test NGN to NGN first, then change currency settings.

## Common Error Codes and Resolution Steps

- **`ERR_FLW_001` (Unauthorized)**: Verify that `FLUTTERWAVE_SECRET_KEY` is correct and doesn't contain spaces.
- **Webhook Not Arriving**: Check `verif-hash` setup. If the platform logs `Missing webhook signature`, ensure the exact case is preserved. Note, ngrok might rewrite headers if not carefully set.
- **`Validation Failed` during transfer**: Ensure amount limits don't exceed Sandbox constraints (usually large numbers > 1,000,000 will be auto-rejected).
