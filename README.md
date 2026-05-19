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

### Sources and routing

Each webhook type supports multiple named instances under `[sources.<type>.<name>]`.
Webhook URL: `POST /webhook/<type>/<name>` (e.g. `/webhook/argocd/prod`).

Routes use full source ids (`argocd.prod`, `cloudflare.main`) and destination ids (`matrix.composite`):

```toml
[sources.argocd.prod]
token = "..."

[sources.argocd.staging]
token = "..."

[[routes]]
sources = ["argocd.prod", "argocd.staging"]
destinations = ["matrix.my_room"]
```

Authentication per source (omit token/secret field to disable):
- **ArgoCD**: `Authorization: Bearer <token>`
- **Cloudflare**: `X-Webhook-Secret: <secret>`
- **Alertmanager**: `X-Token: <token>` or `Authorization: Bearer <token>`

### Environment Variable Overrides
...
## Endpoints

- `POST /webhook/argocd/{name}`
- `POST /webhook/cloudflare/{name}`
- `POST /webhook/alertmanager/{name}`

