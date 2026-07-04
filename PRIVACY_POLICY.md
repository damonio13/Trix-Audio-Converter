# Política de Privacidade — Trix Audio Converter

**Versão:** 1.0.0  
**Última atualização:** 20 de junho de 2025  
**Desenvolvedor:** João Vitor de Melo (joaovmelo259@gmail.com)  
**Conformidade:** Lei Geral de Proteção de Dados (LGPD — Lei nº 13.709/2018)

---

## 1. Introdução

Esta Política de Privacidade descreve como o **Trix Audio Converter** ("Software") coleta, usa, armazena e protege informações dos seus usuários. O Software foi projetado com foco em **privacidade por design** e **minimização de dados**.

## 2. Dados Coletados

### 2.1 Dados que o Software NÃO coleta

O Trix Audio Converter é um aplicativo **offline-first**. Por padrão, o Software **NÃO** coleta:

- ❌ Dados de identificação pessoal (nome, CPF, endereço)
- ❌ Dados de uso ou telemetria
- ❌ Informações do sistema operacional (exceto para logs de crash, veja seção 2.2)
- ❌ Localização geográfica
- ❌ Histórico de conversões (armazenado apenas localmente)

### 2.2 Dados coletados localmente

O Software armazena dados **exclusivamente no dispositivo do usuário**:

| Dado | Finalidade | Retenção |
|------|------------|----------|
| Logs de conversão | Histórico de operações | 30 dias (auto-limpeza) |
| Logs de crash | Diagnóstico de erros fatais | Últimos 20 registros |
| Configurações | Preferências do usuário | Enquanto o Software estiver instalado |
| Fila de arquivos | Estado entre sessões | Enquanto o Software estiver instalado |

### 2.3 Conexões de rede

O Software pode se conectar à internet **apenas** para:

| Serviço | Dados Enviados | Finalidade | Necessita Consentimento |
|---------|---------------|------------|------------------------|
| GitHub Releases | Nenhum | Verificar atualizações | Não (funcionalidade essencial) |
| MusicBrainz | Nome do arquivo/artistas | Lookup de metadados | Não (funcionalidade essencial) |
| Discogs | Nome do arquivo/artistas | Lookup de capa de álbum | Não (funcionalidade essencial) |
| Google Drive / Dropbox | Arquivos selecionados pelo usuário | Sincronização em nuvem | Sim (configuração manual) |

> **Nota:** As conexões com MusicBrainz e Discogs são feitas apenas quando o usuário ativa a busca de metadados. Nenhum dado pessoal é transmitido.

## 3. Base Legal (LGPD)

O tratamento de dados é realizado com base em:

- **Art. 7º, I — Consentimento:** Para funcionalidades opcionais (Google Drive, Discord sync)
- **Art. 7º, II — Execução de contrato:** Para funcionalidades essenciais do Software
- **Art. 7º, IX — Legítimo interesse:** Para melhoria do Software (crash logs anônimos)

## 4. Compartilhamento de Dados

O Software **NÃO** compartilha dados pessoais com terceiros, exceto:

- Quando exigido por lei ou ordem judicial
- Quando necessário para cumprir termos de licença de terceiros (FFmpeg)
- Quando o usuário explicitamente autoriza (sincronização em nuvem)

## 5. Armazenamento e Segurança

### 5.1 Armazenamento local

- Todos os dados são armazenados localmente no dispositivo do usuário
- Localização no Windows: `%LOCALAPPDATA%\trix-audio-converter\`
- Localização em modo portátil: `<diretório_do_executável>\data\`

### 5.2 Segurança

- O Software não transmite dados pessoais por padrão
- Tokens de API são gerados localmente e nunca são compartilhados
- Logs de crash contêm apenas informações técnicas (SO, versão, stack trace)

### 5.3 Criptografia

- Tokens de autenticação internos usam SHA-256
- Conexões HTTPS são utilizadas quando disponíveis (GitHub, MusicBrainz)

## 6. Direitos do Titular (LGPD — Art. 18)

Nos termos da LGPD, você tem direito a:

| Direito | Como exercer |
|---------|-------------|
| **Confirmação** da existência de tratamento | Ver esta política |
| **Acesso** aos dados | Os dados ficam no seu dispositivo |
| **Correção** de dados incompletos | Ajuste nas configurações do Software |
| **Anonimização, bloqueio ou eliminação** | Desinstale o Software e delete a pasta `data/` |
| **Portabilidade** | Exporte seus arquivos manualmente |
| **Eliminação** dos dados tratados com consentimento | Delete a pasta `data/` do Software |
| **Informação** sobre compartilhamento | O Software não compartilha dados |
| **Revogação** do consentimento | Desative funcionalidades opcionais nas configurações |

## 7. Menores de Idade

O Software não é direcionado a menores de 13 anos. Não coletamos intencionalmente dados de menores. Se descobrirmos que coletamos dados de um menor, tomaremos providências imediatas para eliminar essas informações.

## 8. Cookies e Rastreadores

O Software **NÃO** utiliza cookies, pixels de rastreamento, analytics ou qualquer forma de monitoramento de comportamento do usuário.

## 8.1 Service Worker

O Software utiliza um Service Worker para fins de:

- **Cache de arquivos estáticos** (funcionamento offline)
- **NÃO** para rastreamento ou coleta de dados

## 9. Alterações nesta Política

O desenvolvedor reserva-se o direito de atualizar esta Política de Privacidade. Alterações significativas serão comunicadas através:

- Do repositório GitHub: https://github.com/damonio13/Trix-Audio-Converter
- Da documentação incluída com o Software

## 10. Contato

Para exercer seus direitos ou esclarecer dúvidas sobre esta política:

- **Email:** joaovmelo259@gmail.com
- **GitHub Issues:** https://github.com/damonio13/Trix-Audio-Converter/issues
- **LinkedIn:** https://www.linkedin.com/in/jo%C3%A3o-vitor-de-melo-22728a26b/

## 11. Autoridade Nacional de Proteção de Dados (ANPD)

Se você considerar que o tratamento de seus dados viola a LGPD, pode encaminhar reclamação à ANPD:

- **Site:** https://www.gov.br/anpd
- **Email:** encarregado@anpd.gov.br
