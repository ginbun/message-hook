# Messages Hook Server (Rust)

A simple webhook server to forward notifications from ArgoCD and Cloudflare to Matrix and Telegram.

## Features

- **ArgoCD**: Supports standard webhook notifications.
- **Cloudflare**: Supports Cloudflare Alert notifications.
- **Alertmanager**: Supports Prometheus Alertmanager notifications.
- **Targets**: Matrix and Telegram.
- **Security**: Token-based validation for all webhook sources.
- **Configuration**: TOML file with Environment Variable overrides.

## Configuration

Copy `config.toml` and fill in your details.

### Authentication

Each source can be secured with a token/secret:
- **ArgoCD**: Checks `Authorization: Bearer <token>`
- **Cloudflare**: Checks `X-Webhook-Secret: <secret>`
- **Alertmanager**: Checks `X-Token: <token>` or `Authorization: Bearer <token>`

### Environment Variable Overrides
...
## Endpoints

- `POST /webhook/argocd`
- `POST /webhook/cloudflare`
- `POST /webhook/alertmanager`

