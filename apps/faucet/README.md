# ARUNA Public Testnet Faucet (Static Frontend)

## Overview
A web interface allowing developers to request testnet ARU tokens for testing and development.

## Platform Boundaries
- **Stateless Frontend**: Communicates with the stateless Faucet Edge API (`workers/faucet-api`) which handles rate-limiting and token distribution.
- **Verification**: Connects with captcha services (hCaptcha/Turnstile) to mitigate sybil attacks.
