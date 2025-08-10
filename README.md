# ePub Reader Library

Uma biblioteca moderna de livros eletrônicos com capacidades de tradução usando Ollama AI.

## Características

- 📚 **Gerenciamento de Biblioteca**: Organize seus livros EPUB em uma interface moderna
- 🌍 **Tradução Inteligente**: Traduza livros para diferentes idiomas usando Ollama
- 📖 **Leitor Integrado**: Leia seus livros em HTML/CSS/JS otimizado
- 💾 **Armazenamento Local**: Dados salvos em `~/.epubreader/ebooks`
- 🎨 **Interface Moderna**: Design escuro e responsivo

## Pré-requisitos

1. **Rust** (versão 1.70+)
2. **Node.js** (versão 16+)
3. **Ollama** (para funcionalidades de tradução)

### Instalação do Ollama

```bash
# macOS/Linux
curl -fsSL https://ollama.ai/install.sh | sh

# Windows
# Baixe de https://ollama.ai/download

# Inicie o Ollama
ollama serve

# Instale um modelo recomendado
ollama pull llama3.1:8b
```

## Instalação

1. Clone o repositório:
```bash
git clone <repository-url>
cd epub-reader-library
```

2. Instale as dependências do Tauri:
```bash
cargo install tauri-cli
```

3. Execute em modo de desenvolvimento:
```bash
cargo tauri dev
```

4. Para build de produção:
```bash
cargo tauri build
```

## Uso

### Adicionando Livros

1. Clique no botão "Add Book" no canto superior direito
2. Selecione um arquivo EPUB do seu sistema
3. O livro será processado e adicionado à sua biblioteca

### Traduzindo Livros

1. Certifique-se de que o Ollama está rodando
2. Selecione o idioma de destino no seletor do cabeçalho
3. Clique em "Translate" no cartão do livro
4. Aguarde o processo de tradução ser concluído

### Lendo Livros

1. Clique em "Read" no cartão do livro
2. O livro será aberto em seu navegador padrão
3. Use as teclas de seta para navegar entre capítulos
4. Use os controles de leitura para ajustar fonte e tema

## Estrutura do Projeto

```
epub-reader-library/
├── src/                    # Código Rust (backend)
│   ├── main.rs            # Ponto de entrada principal
│   ├── commands.rs        # Comandos Tauri
│   ├── database.rs        # Gerenciamento do banco SQLite
│   ├── epub_processor.rs  # Processamento de arquivos EPUB
│   ├── ollama_client.rs   # Cliente para API do Ollama
│   └── models.rs          # Estruturas de dados
├── src-tauri/             # Configuração do Tauri
├── assets/                # Assets para livros gerados
├── index.html             # Interface principal
├── styles.css             # Estilos da aplicação
├── app.js                 # JavaScript principal
└── README.md              # Este arquivo
```

## Idiomas Suportados

- 🇺🇸 English
- 🇧🇷 Português
- 🇪🇸 Español
- 🇫🇷 Français
- 🇩🇪 Deutsch
- 🇮🇹 Italiano
- 🇯🇵 日本語
- 🇰🇷 한국어
- 🇨🇳 中文
- 🇷🇺 Русский
- 🇸🇦 العربية
- 🇮🇳 हिन्दी

## Armazenamento de Dados

Os dados são armazenados em:
- **macOS**: `~/Library/Application Support/.epubreader/ebooks/`
- **Linux**: `~/.local/share/.epubreader/ebooks/`
- **Windows**: `%APPDATA%\.epubreader\ebooks\`

Cada livro tem:
- Banco SQLite com texto traduzido
- Diretório de imagens extraídas
- Arquivos HTML/CSS/JS gerados

## Desenvolvimento

### Estrutura do Backend (Rust)

- **Database**: SQLite com tabelas para livros, capítulos e imagens
- **EPUB Processing**: Extração de texto, metadados e imagens
- **Ollama Integration**: Cliente HTTP para tradução via IA
- **Tauri Commands**: Interface entre frontend e backend

### Estrutura do Frontend

- **HTML/CSS/JS Vanilla**: Interface simples e rápida
- **Grid Layout**: Visualização de biblioteca em grade
- **Modal System**: Diálogos para status e progresso
- **Responsive Design**: Funciona em diferentes tamanhos de tela

## Contribuindo

1. Fork o projeto
2. Crie uma branch para sua feature (`git checkout -b feature/AmazingFeature`)
3. Commit suas mudanças (`git commit -m 'Add some AmazingFeature'`)
4. Push para a branch (`git push origin feature/AmazingFeature`)
5. Abra um Pull Request

## Licença

Este projeto está licenciado sob a Licença MIT - veja o arquivo [LICENSE](LICENSE) para detalhes.

## Problemas Conhecidos

- A tradução pode ser lenta dependendo do modelo Ollama usado
- Alguns arquivos EPUB com DRM não são suportados
- A interface de leitura é básica (melhorias planejadas)

## Roadmap

- [ ] Suporte para mais formatos (PDF, MOBI)
- [ ] Sincronização em nuvem
- [ ] Anotações e marcadores
- [ ] Leitor integrado na aplicação
- [ ] Temas personalizáveis
- [ ] Estatísticas de leitura