# Homelab Monitor

Monitor de recursos para homelab em terminal, inspirado no btop, desenvolvido em Rust com [Ratatui](https://github.com/ratatui/ratatui) e [Axum](https://github.com/tokio-rs/axum).

Permite acompanhar o status de múltiplos servidores, máquinas virtuais, containers e nós da rede local a partir de um único painel no terminal.

---

## Arquitetura

O sistema é dividido em dois componentes:

1. **`homelab-agent`**: Daemon executado em cada nó a ser monitorado. Coleta métricas de sistema (CPU, memória, swap, discos, rede, uptime e carga média) utilizando `sysinfo` e disponibiliza um endpoint HTTP JSON (`/metrics`).
2. **`homelab-monitor`**: Interface TUI interativa que consulta periodicamente os hosts definidos em `config.toml` e exibe os dados em tempo real com gráficos e indicadores visuais.

---

## Instalação e Execução

### 1. Iniciar o agente nos nós a serem monitorados

Execute o binário do agente no nó remoto ou localmente:

```bash
cargo run --bin homelab-agent
```

Por padrão, o serviço escuta em `0.0.0.0:9191`. A porta pode ser personalizada através da variável de ambiente:

```bash
PORT=9090 cargo run --bin homelab-agent
```

### 2. Configurar os hosts em `config.toml`

Copie o modelo de exemplo para criar o seu `config.toml`:

```bash
cp config.example.toml config.toml
```

Edite o arquivo `config.toml` com os nós do seu homelab:

```toml
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

> O arquivo `config.toml` é ignorado pelo Git para não expor os IPs locais da sua rede.

### 3. Iniciar o Dashboard TUI

```bash
cargo run --bin homelab-monitor
```

---

## Navegação

| Tecla | Ação |
|---|---|
| `q` / `Esc` | Sair |
| `↑` / `k` | Selecionar nó anterior |
| `↓` / `j` / `Tab` | Selecionar próximo nó |
| `r` | Recarregar arquivo de configuração |
| `Ctrl+C` | Encerrar aplicação |
