# 📚 Guia de Uso - ePub Reader Library

## Índice
1. [Instalação e Configuração](#instalação-e-configuração)
2. [Primeira Execução](#primeira-execução)
3. [Interface Principal](#interface-principal)
4. [Adicionando Livros](#adicionando-livros)
5. [Traduzindo Livros](#traduzindo-livros)
6. [Lendo Livros](#lendo-livros)
7. [Configurações Avançadas](#configurações-avançadas)
8. [Solução de Problemas](#solução-de-problemas)
9. [Dicas e Truques](#dicas-e-truques)

---

## 🚀 Instalação e Configuração

### Pré-requisitos
Antes de começar, certifique-se de ter instalado:

- **Rust** (versão 1.70 ou superior)
- **Node.js** (versão 16 ou superior)
- **Ollama** (para funcionalidades de tradução)

### Instalação Automática

1. **Execute o script de instalação:**
   ```bash
   ./setup.sh
   ```
   
   Este script irá:
   - ✅ Verificar se Rust e Node.js estão instalados
   - 📦 Instalar o Tauri CLI
   - 🤖 Instalar e configurar o Ollama
   - 📥 Baixar o modelo de tradução recomendado

### Instalação Manual

Se preferir instalar manualmente:

1. **Instalar Tauri CLI:**
   ```bash
   cargo install tauri-cli --locked
   ```

2. **Instalar Ollama:**
   ```bash
   # macOS/Linux
   curl -fsSL https://ollama.ai/install.sh | sh
   
   # Windows - baixe de https://ollama.ai/download
   ```

3. **Iniciar Ollama:**
   ```bash
   ollama serve
   ```

4. **Instalar modelo de tradução:**
   ```bash
   ollama pull llama3.1:8b
   ```

---

## 🎯 Primeira Execução

### Executando a Aplicação

1. **Modo Desenvolvimento:**
   ```bash
   cargo tauri dev
   ```

2. **Build para Produção:**
   ```bash
   cargo tauri build
   ```

### Verificação do Ollama

Ao iniciar a aplicação pela primeira vez, você verá uma das seguintes situações:

#### ✅ **Ollama Funcionando**
- A aplicação iniciará normalmente
- Você verá a biblioteca vazia
- Funcionalidades de tradução estarão disponíveis

#### ❌ **Ollama Não Detectado**
- Aparecerá um modal: "Ollama Setup Required"
- Opções disponíveis:
  - **"Check Again"**: Verifica novamente se Ollama está rodando
  - **"Continue Without Translation"**: Usa a aplicação sem tradução

---

## 🖥️ Interface Principal

### Layout da Aplicação

```
┌─────────────────────────────────────────────────────────────┐
│ 📚 ePub Reader    [Translate to: English ▼]    [+ Add Book] │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  ┌─────┐  ┌─────┐  ┌─────┐  ┌─────┐                        │
│  │📖   │  │📖   │  │📖   │  │📖   │                        │
│  │Book │  │Book │  │Book │  │Book │                        │
│  │ 1   │  │ 2   │  │ 3   │  │ 4   │                        │
│  └─────┘  └─────┘  └─────┘  └─────┘                        │
│                                                             │
│  ┌─────┐  ┌─────┐  ┌─────┐  ┌─────┐                        │
│  │📖   │  │📖   │  │📖   │  │📖   │                        │
│  │Book │  │Book │  │Book │  │Book │                        │
│  │ 5   │  │ 6   │  │ 7   │  │ 8   │                        │
│  └─────┘  └─────┘  └─────┘  └─────┘                        │
└─────────────────────────────────────────────────────────────┘
```

### Elementos da Interface

#### **Header (Barra Superior)**
- **Logo**: ePub Reader (canto esquerdo)
- **Seletor de Idioma**: "Translate to:" (centro)
- **Botão Add Book**: Adicionar novos livros (direita)

#### **Grid de Livros**
Cada cartão de livro mostra:
- **Capa**: Imagem do livro ou ícone padrão
- **Status de Tradução**: Badge colorido no canto superior direito
- **Título**: Nome do livro
- **Autor**: Nome do autor
- **Idioma**: Idioma original do livro

#### **Status de Tradução**
- 🔘 **Not Started**: Cinza - Tradução não iniciada
- 🟡 **In Progress**: Amarelo - Tradução em andamento
- 🟢 **Completed**: Verde - Tradução concluída
- 🔴 **Failed**: Vermelho - Tradução falhou

---

## 📖 Adicionando Livros

### Processo de Adição

1. **Clique no botão "Add Book"** (canto superior direito)

2. **Selecione um arquivo EPUB:**
   - Navegue pelos seus arquivos
   - Selecione um arquivo com extensão `.epub`
   - Clique em "Abrir"

3. **Processamento Automático:**
   - ⏳ A aplicação mostrará "Loading your library..."
   - 📊 O livro será processado automaticamente:
     - Extração de metadados (título, autor, idioma)
     - Extração da capa
     - Divisão em capítulos
     - Extração de imagens
     - Criação da estrutura no banco de dados

4. **Resultado:**
   - ✅ O livro aparecerá na sua biblioteca
   - 📁 Será criado um diretório em `~/.epubreader/ebooks/[Nome do Livro]/`

### Estrutura Criada

Para cada livro adicionado:

```
~/.epubreader/ebooks/[Nome do Livro]/
├── images/
│   ├── cover.jpg           # Capa extraída
│   ├── chapter_0_0.jpg     # Imagens do capítulo 0
│   ├── chapter_0_1.jpg
│   └── ...
└── [Banco SQLite com texto e metadados]
```

### Formatos Suportados

- ✅ **EPUB**: Totalmente suportado
- ❌ **PDF**: Não suportado (planejado para futuras versões)
- ❌ **MOBI**: Não suportado (planejado para futuras versões)

---

## 🌍 Traduzindo Livros

### Pré-requisitos para Tradução

1. **Ollama deve estar rodando:**
   ```bash
   ollama serve
   ```

2. **Modelo de tradução instalado:**
   ```bash
   ollama pull llama3.1:8b
   ```

### Processo de Tradução

#### **1. Selecionar Idioma de Destino**
- No header, clique no seletor "Translate to:"
- Escolha o idioma desejado da lista

#### **2. Iniciar Tradução**
- Passe o mouse sobre um cartão de livro
- Clique no botão **"Translate"** que aparece

#### **3. Acompanhar Progresso**
- Aparecerá um modal "Translating Book"
- Barra de progresso mostrará o andamento
- Status textual indicará a etapa atual

#### **4. Conclusão**
- ✅ Status mudará para "Completed" (verde)
- 📚 O livro traduzido estará disponível para leitura

### Idiomas Disponíveis

| Código | Nome Nativo | Nome em Inglês |
|--------|-------------|----------------|
| `en` | English | English |
| `pt` | Português | Portuguese |
| `es` | Español | Spanish |
| `fr` | Français | French |
| `de` | Deutsch | German |
| `it` | Italiano | Italian |
| `ja` | 日本語 | Japanese |
| `ko` | 한국어 | Korean |
| `zh` | 中文 | Chinese |
| `ru` | Русский | Russian |
| `ar` | العربية | Arabic |
| `hi` | हिन्दी | Hindi |

### Tempo de Tradução

O tempo varia conforme:
- **Tamanho do livro**: Livros maiores demoram mais
- **Modelo usado**: Modelos maiores são mais lentos mas precisos
- **Hardware**: CPU/GPU mais potentes aceleram o processo

**Estimativas típicas:**
- Livro pequeno (100 páginas): 5-15 minutos
- Livro médio (300 páginas): 15-45 minutos
- Livro grande (500+ páginas): 45+ minutos

---

## 📚 Lendo Livros

### Abrindo um Livro

1. **Passe o mouse sobre um cartão de livro**
2. **Clique no botão "Read"**
3. **O livro abrirá no seu navegador padrão**

### Interface do Leitor

#### **Página Inicial (Capa)**
```
┌─────────────────────────────────────┐
│              [CAPA]                 │
│                                     │
│           Título do Livro           │
│            Nome do Autor            │
│                                     │
│         Índice de Capítulos         │
│         • Capítulo 1               │
│         • Capítulo 2               │
│         • ...                      │
│                                     │
│        [Start Reading]              │
└─────────────────────────────────────┘
```

#### **Página de Capítulo**
```
┌─────────────────────────────────────┐
│ [← Previous] [Contents] [Next →]    │
├─────────────────────────────────────┤
│                                     │
│         Título do Capítulo          │
│                                     │
│  Conteúdo do capítulo com texto     │
│  formatado, parágrafos e imagens    │
│  inseridas nas posições corretas.   │
│                                     │
│  [Imagem do capítulo, se houver]    │
│                                     │
│  Mais texto do capítulo...          │
│                                     │
├─────────────────────────────────────┤
│ ████████████░░░░░░░░░░░░░░░░ 60%    │
└─────────────────────────────────────┘
```

### Controles de Navegação

#### **Teclado**
- **← (Seta Esquerda)**: Capítulo anterior
- **→ (Seta Direita)**: Próximo capítulo
- **Home**: Voltar ao índice
- **Escape**: Voltar ao índice

#### **Mouse/Touch**
- **Botões de navegação**: Previous, Contents, Next
- **Links do índice**: Clique para ir direto ao capítulo

### Controles de Leitura

No canto superior direito de cada capítulo:

```
┌─────────────────┐
│ [A-] [A+] [🌓] [⛶] │
└─────────────────┘
```

- **A-**: Diminuir tamanho da fonte
- **A+**: Aumentar tamanho da fonte
- **🌓**: Alternar tema (claro/escuro)
- **⛶**: Tela cheia

### Recursos Automáticos

#### **Salvamento de Posição**
- ✅ Posição de leitura salva automaticamente
- ✅ Retoma de onde parou ao reabrir
- ✅ Funciona por livro individualmente

#### **Barra de Progresso**
- 📊 Mostra progresso no capítulo atual
- 🔄 Atualiza conforme você rola a página

#### **Responsividade**
- 📱 Funciona em diferentes tamanhos de tela
- 🖥️ Otimizado para desktop e mobile

---

## ⚙️ Configurações Avançadas

### Configuração do Ollama

#### **Modelos Recomendados (em ordem de preferência):**
1. `llama3.1:8b` - Melhor qualidade/velocidade
2. `llama3:8b` - Boa alternativa
3. `llama2:7b` - Mais rápido, qualidade menor
4. `mistral:7b` - Alternativa rápida

#### **Instalando Modelos Específicos:**
```bash
# Modelo recomendado
ollama pull llama3.1:8b

# Modelo mais rápido
ollama pull llama2:7b

# Modelo multilíngue
ollama pull mistral:7b
```

#### **Configurações de Performance:**
```bash
# Para máquinas com mais RAM
ollama pull llama3.1:13b

# Para máquinas com menos recursos
ollama pull llama3.1:8b-q4_0
```

### Configuração de Armazenamento

#### **Localização dos Dados:**
- **macOS**: `~/Library/Application Support/.epubreader/ebooks/`
- **Linux**: `~/.local/share/.epubreader/ebooks/`
- **Windows**: `%APPDATA%\.epubreader\ebooks\`

#### **Estrutura de Dados:**
```
.epubreader/ebooks/
├── library.db              # Banco principal SQLite
├── [Livro 1]/
│   ├── images/             # Imagens extraídas
│   └── html/               # Versão HTML (gerada sob demanda)
├── [Livro 2]/
│   ├── images/
│   └── html/
└── ...
```

### Configuração de Desenvolvimento

#### **Variáveis de Ambiente:**
```bash
# Nível de log
export RUST_LOG=info

# Configuração do Tauri
export TAURI_CONFIG=tauri.conf.json
```

#### **Configuração do Banco:**
O banco SQLite é criado automaticamente com as seguintes tabelas:
- `books` - Metadados dos livros
- `chapters` - Conteúdo dos capítulos
- `images` - Referências de imagens
- `settings` - Configurações da aplicação

---

## 🔧 Solução de Problemas

### Problemas Comuns

#### **1. "Ollama is not running"**

**Sintomas:**
- Modal aparece ao iniciar
- Botão "Translate" não aparece nos livros

**Soluções:**
```bash
# Verificar se Ollama está instalado
ollama --version

# Iniciar Ollama
ollama serve

# Verificar se está rodando
curl http://localhost:11434/api/tags
```

#### **2. "No suitable model available"**

**Sintomas:**
- Tradução falha imediatamente
- Erro sobre modelo não encontrado

**Soluções:**
```bash
# Instalar modelo recomendado
ollama pull llama3.1:8b

# Verificar modelos instalados
ollama list

# Testar modelo
ollama run llama3.1:8b "Hello, how are you?"
```

#### **3. Livro não aparece após adicionar**

**Sintomas:**
- Arquivo EPUB selecionado mas não aparece na biblioteca
- Loading infinito

**Soluções:**
1. **Verificar formato do arquivo:**
   - Certifique-se que é um arquivo `.epub` válido
   - Teste abrir em outro leitor de EPUB

2. **Verificar logs:**
   ```bash
   # Executar com logs detalhados
   RUST_LOG=debug cargo tauri dev
   ```

3. **Verificar permissões:**
   - Certifique-se que a aplicação pode escrever em `~/.epubreader/`

#### **4. Tradução muito lenta**

**Sintomas:**
- Tradução demora horas
- Sistema fica lento durante tradução

**Soluções:**
1. **Usar modelo menor:**
   ```bash
   ollama pull llama2:7b
   ```

2. **Verificar recursos do sistema:**
   - RAM disponível (mínimo 8GB recomendado)
   - CPU usage durante tradução

3. **Configurar Ollama:**
   ```bash
   # Limitar uso de CPU
   export OLLAMA_NUM_PARALLEL=1
   ```

#### **5. Erro ao abrir livro para leitura**

**Sintomas:**
- Clique em "Read" não abre nada
- Erro no navegador

**Soluções:**
1. **Verificar navegador padrão:**
   - Certifique-se que há um navegador configurado

2. **Verificar arquivos HTML:**
   ```bash
   # Verificar se HTML foi gerado
   ls ~/.epubreader/ebooks/[Nome do Livro]/html/
   ```

3. **Regenerar HTML:**
   - Remova a pasta `html` do livro
   - Clique em "Read" novamente

### Logs e Debugging

#### **Ativar Logs Detalhados:**
```bash
# Desenvolvimento
RUST_LOG=debug cargo tauri dev

# Produção
RUST_LOG=info ./target/release/epub-reader-library
```

#### **Localização dos Logs:**
- **Console**: Durante desenvolvimento
- **Sistema**: Logs do sistema operacional
- **Arquivo**: (se configurado) `~/.epubreader/logs/`

### Limpeza e Manutenção

#### **Limpar Cache:**
```bash
# Remover arquivos HTML gerados
rm -rf ~/.epubreader/ebooks/*/html/

# Limpar cache do Ollama
ollama rm [modelo]
ollama pull [modelo]
```

#### **Reset Completo:**
```bash
# CUIDADO: Remove todos os dados
rm -rf ~/.epubreader/
```

---

## 💡 Dicas e Truques

### Otimização de Performance

#### **1. Escolha do Modelo Ollama**
- **Para velocidade**: `llama2:7b` ou `mistral:7b`
- **Para qualidade**: `llama3.1:8b` ou `llama3.1:13b`
- **Para idiomas específicos**: Modelos especializados

#### **2. Gerenciamento de Espaço**
- Traduções ocupam espaço adicional
- Considere traduzir apenas livros que vai ler
- Remova traduções antigas se necessário

#### **3. Organização da Biblioteca**
- Use nomes descritivos para arquivos EPUB
- Organize por autor/série antes de importar
- Considere criar backups dos arquivos originais

### Fluxo de Trabalho Recomendado

#### **Para Leitura Casual:**
1. Adicione livros conforme necessário
2. Traduza apenas quando for ler
3. Use modelo rápido (`llama2:7b`)

#### **Para Leitura Intensiva:**
1. Adicione vários livros de uma vez
2. Configure tradução em lote (futuro)
3. Use modelo de alta qualidade (`llama3.1:8b`)

#### **Para Desenvolvimento:**
1. Use modo desenvolvimento (`cargo tauri dev`)
2. Ative logs detalhados (`RUST_LOG=debug`)
3. Teste com livros pequenos primeiro

### Atalhos e Produtividade

#### **Navegação Rápida:**
- Use teclas de seta para navegar entre capítulos
- `Home` para voltar ao índice rapidamente
- `Escape` como alternativa ao `Home`

#### **Leitura Confortável:**
- Ajuste o tamanho da fonte antes de começar
- Use modo tela cheia para imersão
- Alterne tema conforme iluminação ambiente

#### **Gerenciamento de Traduções:**
- Traduza livros em ordem de prioridade
- Monitore o progresso pelo status colorido
- Cancele traduções desnecessárias

### Integração com Outros Sistemas

#### **Backup e Sincronização:**
```bash
# Backup da biblioteca
tar -czf biblioteca-backup.tar.gz ~/.epubreader/

# Restaurar backup
tar -xzf biblioteca-backup.tar.gz -C ~/
```

#### **Exportação de Dados:**
- Arquivos HTML podem ser copiados
- Banco SQLite pode ser consultado diretamente
- Imagens ficam organizadas por livro

---

## 📞 Suporte e Comunidade

### Reportar Problemas

1. **Verifique este guia primeiro**
2. **Colete informações:**
   - Versão do sistema operacional
   - Versão do Rust/Tauri
   - Logs de erro
   - Passos para reproduzir

3. **Abra uma issue no repositório**

### Contribuições

- Fork do projeto
- Implemente melhorias
- Teste thoroughly
- Abra Pull Request

### Roadmap Futuro

- [ ] Suporte para PDF e MOBI
- [ ] Sincronização em nuvem
- [ ] Anotações e marcadores
- [ ] Leitor integrado na aplicação
- [ ] Temas personalizáveis
- [ ] Estatísticas de leitura
- [ ] Tradução em lote
- [ ] Suporte offline melhorado

---

## 📝 Conclusão

O ePub Reader Library é uma ferramenta poderosa para gerenciar e traduzir sua biblioteca digital. Com este guia, você deve conseguir:

- ✅ Instalar e configurar a aplicação
- ✅ Adicionar e organizar seus livros
- ✅ Traduzir livros para seu idioma preferido
- ✅ Ler confortavelmente com controles avançados
- ✅ Resolver problemas comuns
- ✅ Otimizar sua experiência de leitura

**Aproveite sua nova biblioteca digital inteligente!** 📚✨

---

*Última atualização: Dezembro 2024*
*Versão do Guia: 1.0*