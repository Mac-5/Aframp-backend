# M-Pesa Integration Guide

## Developer Portal Setup and App Registration

1. **Sign Up**: Navigate to the [Daraja (Safaricom Developer Portal)](https://developer.safaricom.co.ke/).
2. **Create a Test App**: Go to **My Apps** > **Create New App**.
3. **Select Products**: Tick the boxes for **Lipa na M-Pesa Sandbox** (STK Push) and **B2C Sandbox** (if allowing withdrawals).
4. **App Credentials**: Once created, click on the app to retrieve your **Consumer Key** and **Consumer Secret**.
5. **Sandbox Credentials**: Go to the **APIs** section to retrieve your test Passkey and test business shortcodes (e.g., `174379`).

## Required Environment Variables

You must maintain the Daraja credentials in `.env` string format or mapped config file:

```env
# Required for Daraja OAuth token generation
MPESA_CONSUMER_KEY=your_consumer_key_here
MPESA_CONSUMER_SECRET=your_consumer_secret_here
# Used for Lipa Na M-Pesa STK push generation
MPESA_PASSKEY=bfb279f9aa9bdbcf158e97dd71a467cd2e0c893059b10f78e6b72ada1ed2c919
```

## Callback URL Configuration

M-Pesa relies heavily on asynchronous callbacks.

### For STK Push (Pay-in)
When initiating an STK Push `POST /mpesa/stkpush/v1/processrequest`, you must supply a `CallBackURL`.
Example: `https://api.yourdomain.com/webhooks/mpesa/stk`

### For B2C (Withdrawals)
When registering URLs for B2C (`POST /mpesa/b2c/v1/paymentrequest`), you supply two URLs:
- `ResultURL`: Where success/failure payloads are delivered. Example: `https://api.yourdomain.com/webhooks/mpesa/b2c/result`
- `QueueTimeOutURL`: Where timeout responses are delivered. Example: `https://api.yourdomain.com/webhooks/mpesa/b2c/timeout`

## Sandbox Testing Procedures

### Simulating STK Push Confirmations
The Daraja sandbox requires a real test number linked to the sandbox. When you trigger an STK Push, it behaves like a live transaction but utilizes test funds. Provide a simulated success timestamp and amount.
- **Tip**: Safaricom provides test MSISDNs (like `254708374149`) to test End-to-End.

### Simulating B2C Withdrawals
To test withdrawals, generate B2C API calls to a designated test number. The Daraja portal simulates backend processing and eventually sends a callback to your registered `ResultURL`. It will mock a successful or failed response based on predefined test conditions (e.g., specific test numbers trigger insufficient funds).

## Known M-Pesa Sandbox Limitations and Workarounds

- **IP Allowlisting Limitations**: Safaricom's B2C callbacks require the backend server to have a static public IP. If testing locally, tools like ngrok will generally work for STK push, but B2C might require explicit IP registration.
- **Inconsistent Callbacks**: Daraja sandbox callbacks can sometimes be heavily delayed or outright dropped during Safaricom maintenance windows. Use a fallback query mechanism (`/mpesa/stkpushquery/v1/query`) if callbacks take longer than 90 seconds.
- **Passkey Management**: The Sandbox passkey rarely changes, but in production, any portal reset will require an update to `MPESA_PASSKEY`.

## Common Error Codes and Resolution Steps

- **`401 Unauthorized` / Invalid Token**: Safaricom auth tokens expire every 3600 seconds. Ensure the token is being re-fetched or cached correctly before issuing an STK or B2C call. Check `MPESA_CONSUMER_KEY` and `MPESA_CONSUMER_SECRET`.
- **`1032 Request cancelled by user`**: The user canceled the STK push prompt. Treat this as a failure state and log it appropriately.
- **`Timeout`**: The STK request went unfulfilled (usually phone is dead, or ignoring the prompt). Validate via STK push query.
- **Invalid Callback URL Format**: URLs must be securely hosted via an actual HTTPS configuration and cannot contain IP address literals during registration.
