<div align="center">

# Trix Audio Converter

### Conversor de Áudio Universal para Desktop

![Version](https://img.shields.io/badge/version-1.0.0-blue?style=flat-square)
![License](https://img.shields.io/badge/license-MIT-green?style=flat-square)
![Rust](https://img.shields.io/badge/backend-Rust-orange?style=flat-square)
![React](https://img.shields.io/badge/frontend-React%20%2B%20TypeScript-61DAFB?style=flat-square)
![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20macOS%20%7C%20Linux-lightgrey?style=flat-square)

[![CI](https://github.com/damonio13/Trix-Audio-Converter/actions/workflows/ci.yml/badge.svg)](https://github.com/damonio13/Trix-Audio-Converter/actions/workflows/ci.yml)
[![GitHub release](https://img.shields.io/github/v/release/damonio13/Trix-Audio-Converter?style=flat-square)](https://github.com/damonio13/Trix-Audio-Converter/releases)

**106 formatos de saída** | **83 formatos de entrada** | **Interface glassmorphism** | **Offline-first**

---

[Instalação](#instalação) · [Funcionalidades](#funcionalidades) · [CLI](#cli) · [Contribuir](#contribuir) · [Licença](#licença)

</div>

---

## Visão Geral

O **Trix Audio Converter** é um conversor de áudio desktop de alta performance construído com **Rust** no backend e **React + TypeScript** no frontend. Projetado para conversão em lote, oferece suporte a mais de 100 formatos de áudio com processamento paralelo multi-thread.

```
┌─────────────────────────────────────────────────────┐
│  Trix Audio Converter                               │
│  ┌─────────┐  ┌──────────┐  ┌──────────────────┐   │
│  │ Rust    │  │ Axum     │  │ React + Vite     │   │
│  │ Backend │→ │ HTTP API │→ │ Glassmorphism UI │   │
│  └─────────┘  └──────────┘  └──────────────────┘   │
│  ┌──────────────────────────────────────────────┐   │
│  │  FFmpeg  •  Multi-thread  •  Offline-first   │   │
│  └──────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────┘
```

## Instalação

### Windows (Recomendado)

```bash
# Baixe o instalador .exe na página de Releases
# https://github.com/damonio13/Trix-Audio-Converter/releases

# Ou compile a partir do código-fonte:
git clone https://github.com/damonio13/Trix-Audio-Converter.git
cd Trix-Audio-Converter
cargo build --release --manifest-path src-rs/Cargo.toml
```

### macOS / Linux

```bash
git clone https://github.com/damonio13/Trix-Audio-Converter.git
cd Trix-Audio-Converter

# Backend
cargo build --release --manifest-path src-rs/Cargo.toml

# Frontend
npm install
npm run build
```

### Requisitos

| Componente | Versão Mínima |
|------------|---------------|
| **Rust** | 1.70+ |
| **Node.js** | 18+ |
| **FFmpeg** | Qualquer versão recente (no PATH do sistema) |

## Funcionalidades

### Conversão de Áudio

| Recurso | Descrição |
|---------|-----------|
| **106 formatos** | MP3, FLAC, WAV, AAC, OGG, Opus, WMA, ALAC, AIFF, APE, WV, TTA, DSD e mais |
| **83 formatos de entrada** | Todos os formatos de saída + MKV, MP4, AVI, MOV, MID, S3M |
| **Multi-thread** | Conversão paralela utilizing todos os cores da CPU |
| **Codec Copy** | Cópia direta do stream sem re-encode quando possível |
| **Normalização** | Loudnorm -16 LUFS integrado |
| **Preservação de tags** | Metadados ID3/Vorbis preservados na conversão |

### Efeitos de Áudio

```
Bass Boost     ▓▓▓▓▓▓░░░░  -20 a +20 dB
Treble Boost   ▓▓▓▓▓▓░░░░  -20 a +20 dB
Reverb         ▓▓▓░░░░░░░  0 - 100%
Velocidade     ▓▓▓▓▓░░░░░  0.5x - 2.0x
Pitch          ▓▓▓▓▓▓▓░░░  -12 a +12 semitones
Compressor     ▓▓▓▓░░░░░░  Configurável
Chorus         ▓▓▓░░░░░░░  Configurável
Flanger        ▓▓░░░░░░░░  Configurável
+ 11 presets de efeitos profissionais
```

### Interface

- **Glassmorphism** — UI moderna com blur e transparências
- **5 temas** — Space Violet, Aurora Boreal, Tokyo Cyberpunk, Sunset Obsidian, Emerald Forest
- **10 idiomas** — PT-BR, EN, ES, FR, DE, JA, RU, AR, HI, ZH-CN
- **Drag-and-drop** — Arraste arquivos e pastas diretamente
- **Mini player** — Preview com waveform integrado
- **Comparação A/B** — Compare antes e depois da conversão
- **Acessibilidade** — Navegação por teclado, ARIA labels, foco visível

### Ferramentas

| Ferramenta | Descrição |
|------------|-----------|
| **Cortar/Trim** | Defina início e fim da conversão |
| **Renomear em lote** | Padrões: `{name}`, `{n}`, `{artist}`, `{album}`, `{title}`, `{date}` |
| **Conversão agendada** | Agendamento por datetime ou delay |
| **Extrair áudio de vídeo** | MP4, MKV, AVI → áudio |
| **Extrair capa de álbum** | Busca automática em MusicBrainz + Discogs |
| **Persistência da fila** | Estado preservado entre sessões |
| **Ações pós-conversão** | Abrir pasta, alerta sonoro, desligar, hibernar |
| **Sincronização em nuvem** | Google Drive e Dropbox |
| **Auto-update** | Atualizações via GitHub Releases |
| **Crash logs** | Relatórios de erro automáticos |

## Arquitetura

```
Trix-Audio-Converter/
├── src/                        # Frontend React + TypeScript
│   ├── components/
│   │   ├── layout/             # TitleBar, Sidebar
│   │   ├── panels/             # QueuePanel, FormatSelector, SettingsPanel
│   │   └── ui/                 # Modal, ErrorBoundary
│   ├── hooks/                  # useI18n, useQueue, useOnline
│   ├── utils/                  # api.ts, format.ts
│   └── types/                  # TypeScript definitions
├── src-rs/                     # Backend Rust
│   └── src/
│       ├── main.rs             # Entry point (tao+wry)
│       ├── api.rs              # Servidor HTTP (axum)
│       ├── cli.rs              # CLI (clap)
│       ├── converter.rs        # Motor de conversão paralelo
│       ├── formats.rs          # 106 formatos de saída
│       ├── effects.rs          # Efeitos de áudio
│       ├── metadata.rs         # Leitura/escrita de tags
│       ├── album_art.rs        # Busca de capas
│       ├── crash_logger.rs     # Logs de crash automáticos
│       └── ...
├── public/                     # Service Worker, manifest
├── assets/                     # Ícones
├── dist/                       # Build de produção
└── .github/workflows/          # CI/CD (GitHub Actions)
```

## CLI

```bash
# Conversão básica
trix -i ~/Music -o ~/Converted -f mp3

# Com efeitos
trix -i ./songs -o ./out -f mp3 --bass-boost 5 --reverb 20
trix -i ./podcast -o ./out -f mp3 --effect-preset podcast

# Listar informações
trix --list-formats
trix --list-codecs
trix --list-effects

# Modo GUI (padrão)
trix --gui
```

## Desenvolvimento

```bash
# Frontend (dev server)
npm install
npm run dev

# Backend (Rust)
cd src-rs
cargo run

# Lint & Type Check
npm run lint
npx tsc --noEmit

# Build completo
npm run build
cargo build --release --manifest-path src-rs/Cargo.toml
```

## CI/CD

O projeto utiliza **GitHub Actions** para:

- **Teste** — Ubuntu, Windows, macOS
- **Build** — Binários para todas as plataformas
- **Release** — Publicação automática no GitHub Releases

## Variáveis de Ambiente

Copie `.env.example` para `.env` e configure conforme necessário. Veja `.env.example` para todas as opções disponíveis.

## Documentação Legal

Veja o arquivo [`LEGAL.md`](LEGAL.md) na raiz do projeto para acessar:
* **Termos de Uso**
* **Política de Privacidade (LGPD) — Offline-first**
* **Licença de Software (MIT)**

## Contribuir

Contribuições são bem-vindas! Veja como:

1. Fork o projeto
2. Crie uma branch de feature (`git checkout -b feature/nova-feature`)
3. Commit suas mudanças (`git commit -m 'Adiciona nova feature'`)
4. Push para a branch (`git push origin feature/nova-feature`)
5. Abra um Pull Request

## Contato

**João Vitor de Melo**

- 📧 Email: [joaovmelo259@gmail.com](mailto:joaovmelo259@gmail.com)
- 💼 LinkedIn: [linkedin.com/in/joão-vitor-de-melo](https://www.linkedin.com/in/jo%C3%A3o-vitor-de-melo-22728a26b/)
- 🐙 GitHub: [github.com/damonio13](https://github.com/damonio13)

## Licença

Este projeto é licenciado sob a [MIT License](LEGAL.md).

```
MIT License — Copyright (c) 2026 Trix Audio Converter
Developed by João Vitor de Melo
```

---

<div align="center">

Feito com ❤️ e Rust

</div>
