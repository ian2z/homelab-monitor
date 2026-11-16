# ⚡ Homelab Monitor

Um monitor de homelab leve para terminal no estilo **BTOP**, construído em Rust com [Ratatui](https://github.com/ratatui/ratatui) e [Axum](https://github.com/tokio-rs/axum).

Permite monitorar múltiplos servidores, VMs, Raspberry Pis e containers na sua rede local em tempo real diretamente do seu terminal.

---

## 🏗️ Arquitetura

O projeto é dividido em dois componentes leves:

1. **`homelab-agent`**: Daemon ultraleve (~5MB de RAM) que roda em cada máquina do seu homelab coletando métricas (CPU, RAM, Swap, Discos, Rede, Uptime, Load Average) via `sysinfo` e expondo um endpoint HTTP JSON (`/metrics`).
2. **`homelab-monitor`**: O dashboard TUI interativo em Ratatui. Ele lê a lista de hosts no arquivo `config.toml`, faz polling assíncrono em paralelo e renderiza as métricas com gráficos Sparkline e medidores visuais.

---

## 🚀 Como Usar

### 1. Iniciar o Agente nas máquinas que deseja monitorar

Na máquina (ou localmente para teste):
```bash
cargo run --bin homelab-agent
```
> Por padrão escuta em `0.0.0.0:9191`. Você pode mudar a porta com a variável de ambiente `PORT=9999`.

### 2. Configurar os nós no `config.toml`

Crie ou edite o arquivo `config.toml` na raiz do projeto:

```toml
# Intervalo de atualização em milissegundos
refresh_rate_ms = 1000

[[nodes]]
name = "Servidor Principal"
url = "http://127.0.0.1:9191"

[[nodes]]
name = "Raspberry Pi NAS"
url = "http://192.168.1.15:9191"

[[nodes]]
name = "Proxmox Node"
url = "http://192.168.1.50:9191"
```

### 3. Iniciar o Dashboard TUI

```bash
cargo run --bin homelab-monitor
# ou simplesmente:
cargo run
```

---

## ⌨️ Atalhos de Teclado no Dashboard

| Tecla | Ação |
|---|---|
| `q` ou `Esc` | Sair do monitor |
| `↑` / `k` | Selecionar nó anterior |
| `↓` / `j` / `Tab` | Selecionar próximo nó |
| `r` | Recarregar `config.toml` |
| `Ctrl+C` | Encerrar |
