# Webhook Testing Guide

This guide details exactly how to configure, test, and debug payment provider webhooks natively on your local development machine. Webhook payloads inform the backend of external payment events (like `charge.completed` or `transfer.success`), which requires your workstation to be publicly accessible.

## Local Webhook Forwarding with ngrok

During local development, your `localhost:8000` is hidden behind a NAT/Firewall.

1. **Install ngrok**: Follow instructions on [ngrok.com](https://ngrok.com/download)
2. **Expose your local server**: Run the following command:
   ```bash
   ngrok http 8000
   ```
3. **Capture your public URL**: Note the `Forwarding` URL. Example: `https://a1b2-c3-d4.ngrok-free.app`
4. **Update Provider Dashboards**:
   - Go to your Flutterwave or Paystack dashboard (Test Mode).
   - Set the webhook URL to: `https://a1b2-c3-d4.ngrok-free.app/webhooks/paystack` (or `/flutterwave`).
5. **Restart Server**: Keep ngrok active and start your backend platform server locally. Any incoming paystack event to your ngrok URL will hit your local backend seamlessly.

> **Note**: For M-Pesa B2C callback testing, Safaricom may sometimes reject ngrok domains. In such scenarios, use a hosted staging environment instead or an explicitly allowlisted IP. 

## Replaying Webhook Events

Providers allow you to easily replay webhooks from the dashboard, saving you from generating new test payments iteratively.

### Replay via Paystack
1. Navigate to **Transactions** in Test Mode.
2. Select a transaction you want to replay.
3. Click on the **Webhook** tab inside the transaction detail pop-out.
4. Click **Resend Webhook**. Your local `ngrok` output window will display a `200 OK` if successfully received.

### Replay via Flutterwave
1. Navigate to **Transactions** > **Test Transactions**.
2. Select any transfer or payment.
3. Choose the **Webhook/Logs** action to simulate the event again.

## Verifying Signature Validation Locally

To test local validation without relying directly on a provider's dashboard replay, you can craft specific `curl` commands simulating the headers and payload.

### Validating Paystack Signature Locally

Generate your HMAC payload in the terminal via OpenSSL, then send it. (Assuming `PAYSTACK_WEBHOOK_SECRET=sk_test_mocked`):

```bash
# Provide payload JSON
PAYLOAD='{"event":"charge.success","data":{"amount":5000}}'

# Create HMAC Hash
SIGNATURE=$(echo -n $PAYLOAD | openssl dgst -sha512 -hmac "sk_test_mocked" | awk '{print $2}')

# Send Local cURL matching the platform
curl -X POST http://localhost:8000/webhooks/paystack \
  -H "Content-Type: application/json" \
  -H "x-paystack-signature: $SIGNATURE" \
  -d "$PAYLOAD"
```
Expect a `200 OK`. If you change the payload or don't generate the hash correctly, expect an `InvalidSignature` error output on your terminal console.

### Validating Flutterwave Signature Locally

Flutterwave looks for a statically matched header instead of a dynamic HMAC hash. (Assuming `FLUTTERWAVE_WEBHOOK_SECRET=mycustom_secrethash123`):

```bash
curl -X POST http://localhost:8000/webhooks/flutterwave \
  -H "Content-Type: application/json" \
  -H "verif-hash: mycustom_secrethash123" \
  -d '{"event":"charge.completed","data":{"amount":100}}'
```
Expect a `200 OK`. Altering `verif-hash` to `wrong_sec` should return a signature failure explicitly blocked by the platform.
