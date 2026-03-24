# Provider Feature Comparison

This table outlines the supported features, currencies, and operating countries for each payment provider configured in the platform.

| Feature | Flutterwave | Paystack | M-Pesa |
|---------|-------------|----------|--------|
| **Core Operations** | | | |
| Collect Payments (Pay-in) | ✅ Yes | ✅ Yes | ✅ Yes (STK Push) |
| Payouts / Withdrawals (Pay-out) | ✅ Yes | ✅ Yes | ✅ Yes (B2C) |
| Webhook Support | ✅ Yes | ✅ Yes | ✅ Yes (Callback URLs) |
| Hosted Checkout | ✅ Yes | ✅ Yes | ❌ No (Direct STK) |
| **Common Currencies Supported** | | | |
| NGN (Nigerian Naira) | ✅ Yes | ✅ Yes | ❌ No |
| GHS (Ghanaian Cedi) | ✅ Yes | ✅ Yes | ❌ No |
| KES (Kenyan Shilling) | ✅ Yes | ✅ Yes | ✅ Yes |
| USD (US Dollar) | ✅ Yes | ✅ Yes | ❌ No |
| ZAR (South African Rand) | ✅ Yes | ✅ Yes | ❌ No |
| **Primary Countries** | Nigeria, Kenya, Ghana, South Africa, etc. (Pan-African) | Nigeria, Ghana, South Africa, Kenya | Kenya, Tanzania, etc. |
| **Authentication Logic** | Bearer Token / Secret Key | Bearer Token / Secret Key | Basic Auth (generating short-lived token) + Passkey for STK |
| **Webhook Signature Validation**| HMAC SHA256 (via verif-hash custom HTTP header) | HMAC SHA512 (via x-paystack-signature header) | Direct Server IP safelisting or basic certs (Often relying on internal state mapping) |
| **Test Environment Limits** | Cards often simulated via specific test numbers. Bank transfer simulation requires specific test endpoints. | Extensive test card support. Webhooks can be manually triggered in dashboard. | Sandbox requires short-lived STK push simulation on Safaricom developer portal. |
