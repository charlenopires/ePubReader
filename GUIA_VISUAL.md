# 🎨 Guia Visual - ePub Reader Library

## 📱 Interface Principal

### Tela Inicial (Biblioteca Vazia)
```
┌─────────────────────────────────────────────────────────────────────────────┐
│ 📚 ePub Reader Library    [Translate to: Select Language ▼]    [+ Add Book] │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│                                    📚                                       │
│                                                                             │
│                            Your Library is Empty                           │
│                                                                             │
│                        Add your first ebook to get started                 │
│                                                                             │
│                            [Add Your First Book]                           │
│                                                                             │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Biblioteca com Livros
```
┌─────────────────────────────────────────────────────────────────────────────┐
│ 📚 ePub Reader Library    [Translate to: Português ▼]        [+ Add Book]   │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐        │
│  │     📖      │  │     📖      │  │     📖      │  │     📖      │        │
│  │             │  │             │  │             │  │             │        │
│  │   [CAPA]    │  │   [CAPA]    │  │   [CAPA]    │  │   [CAPA]    │        │
│  │             │  │             │  │             │  │             │        │
│  │             │  │             │  │             │  │             │        │
│  │🟢 COMPLETED │  │🟡 IN PROGRESS│  │🔘 NOT STARTED│  │🔴 FAILED    │        │
│  │             │  │             │  │             │  │             │        │
│  │ Dom Casmurro│  │ 1984        │  │ Dune        │  │ Neuromancer │        │
│  │ Machado de  │  │ George      │  │ Frank       │  │ William     │        │
│  │ Assis       │  │ Orwell      │  │ Herbert     │  │ Gibson      │        │
│  │ Portuguese  │  │ English     │  │ English     │  │ English     │        │
│  └─────────────┘  └─────────────┘  └─────────────┘  └─────────────┘        │
│                                                                             │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐        │
│  │     📖      │  │     📖      │  │     📖      │  │     📖      │        │
│  │   [CAPA]    │  │   [CAPA]    │  │   [CAPA]    │  │   [CAPA]    │        │
│  │🟢 COMPLETED │  │🔘 NOT STARTED│  │🟡 IN PROGRESS│  │🟢 COMPLETED │        │
│  │ The Hobbit  │  │ Fundação    │  │ Cem Anos de │  │ Harry Potter│        │
│  │ J.R.R.      │  │ Isaac       │  │ Solidão     │  │ J.K.        │        │
│  │ Tolkien     │  │ Asimov      │  │ G. G. Márquez│  │ Rowling     │        │
│  │ English     │  │ English     │  │ Spanish     │  │ English     │        │
│  └─────────────┘  └─────────────┘  └─────────────┘  └─────────────┘        │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Hover sobre Livro (Ações Disponíveis)
```
┌─────────────┐
│     📖      │
│             │
│   [CAPA]    │
│             │
│ ┌─────────┐ │ ← Overlay aparece no hover
│ │ [Read]  │ │
│ │[Translate]│ │
│ └─────────┘ │
│🟢 COMPLETED │
│ Dom Casmurro│
│ Machado de  │
│ Assis       │
│ Portuguese  │
└─────────────┘
```

## 🔄 Modais e Diálogos

### Modal: Ollama Setup Required
```
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│                    ┌─────────────────────────────────┐                      │
│                    │                                 │                      │
│                    │        Ollama Setup Required    │                      │
│                    │                                 │                      │
│                    │  Ollama is not running. Please │                      │
│                    │  start Ollama to enable        │                      │
│                    │  translation features.         │                      │
│                    │                                 │                      │
│                    │  [Check Again] [Continue Without│                      │
│                    │                 Translation]    │                      │
│                    │                                 │                      │
│                    └─────────────────────────────────┘                      │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Modal: Tradução em Progresso
```
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│                    ┌─────────────────────────────────┐                      │
│                    │                                 │                      │
│                    │        Translating Book         │                      │
│                    │                                 │                      │
│                    │ ████████████░░░░░░░░░░░░░░░░░░░ │                      │
│                    │              65%                │                      │
│                    │                                 │                      │
│                    │   Translating chapter 12 of 18 │                      │
│                    │                                 │                      │
│                    │           [Cancel]              │                      │
│                    │                                 │                      │
│                    └─────────────────────────────────┘                      │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Loading: Carregando Biblioteca
```
┌─────────────────────────────────────────────────────────────────────────────┐
│ 📚 ePub Reader Library    [Translate to: Select Language ▼]    [+ Add Book] │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│                                    ⟳                                       │
│                                                                             │
│                            Loading your library...                         │
│                                                                             │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

## 📖 Interface do Leitor

### Página Inicial do Livro (Capa)
```
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│                                                                             │
│                              ┌─────────────┐                               │
│                              │             │                               │
│                              │    CAPA     │                               │
│                              │   DO LIVRO  │                               │
│                              │             │                               │
│                              └─────────────┘                               │
│                                                                             │
│                              Dom Casmurro                                   │
│                             Machado de Assis                                │
│                                                                             │
│                           Table of Contents                                 │
│                                                                             │
│                          • Capítulo I - Do título                          │
│                          • Capítulo II - Do livro                          │
│                          • Capítulo III - A denúncia                       │
│                          • Capítulo IV - A ideia                           │
│                          • ...                                             │
│                                                                             │
│                            [Start Reading]                                  │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Página de Capítulo
```
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│                    Capítulo I - Do título                                   │
│                                                                             │
│  [← Previous]              [Contents]              [Next →]                │
│                                                                             │
│ ┌─────────────────┐                                                         │
│ │ [A-] [A+] [🌓] [⛶] │ ← Controles de leitura                                │
│ └─────────────────┘                                                         │
│                                                                             │
│     Uma noite destas, vindo da cidade para o Engenho Novo, encontrei       │
│ no trem da Central um rapaz aqui do bairro, que eu conheço de vista e      │
│ de chapéu. Cumprimentou-me, sentou-se ao pé de mim, falou da lua e dos     │
│ ministros, e acabou recitando-me versos. A viagem era curta, e os versos   │
│ pode ser que não fossem inteiramente maus. Faltava-lhes, porém, o último   │
│ acabamento, o lavor do artista, como se diz. Eram versos de um homem       │
│ feliz.                                                                      │
│                                                                             │
│     No dia seguinte entrei a pensar na vida do meu vizinho, e perguntei    │
│ a mim mesmo se nunca fora poeta. Pareceu-me que sim e não. Há muito que    │
│ não leio versos; mas a sensação que tive é que a poesia desta vez era      │
│ diferente da de outros tempos.                                              │
│                                                                             │
│                              [Imagem, se houver]                            │
│                                                                             │
│     Continuação do texto do capítulo...                                    │
│                                                                             │
├─────────────────────────────────────────────────────────────────────────────┤
│ ████████████████████████████████████████░░░░░░░░░░░░░░░░░░░░░░░░░░░░ 75%    │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Controles de Leitura (Expandido)
```
┌─────────────────────────────────────────────────────────────────────────────┐
│                                                                             │
│ ┌─────────────────┐                                                         │
│ │ [A-] [A+] [🌓] [⛶] │ ← Clique em cada botão                                │
│ └─────────────────┘                                                         │
│   │    │    │    │                                                          │
│   │    │    │    └── Tela cheia                                             │
│   │    │    └─────── Alternar tema (claro/escuro)                           │
│   │    └──────────── Aumentar fonte                                         │
│   └───────────────── Diminuir fonte                                         │
│                                                                             │
│ Efeito dos controles:                                                       │
│                                                                             │
│ Fonte pequena:  Este é um texto de exemplo                                 │
│ Fonte normal:   Este é um texto de exemplo                                 │
│ Fonte grande:   Este é um texto de exemplo                                 │
│                                                                             │
│ Tema claro:  ┌─────────────────┐                                           │
│              │ Texto em preto  │                                           │
│              │ Fundo branco    │                                           │
│              └─────────────────┘                                           │
│                                                                             │
│ Tema escuro: ┌─────────────────┐                                           │
│              │ Texto em branco │                                           │
│              │ Fundo preto     │                                           │
│              └─────────────────┘                                           │
└─────────────────────────────────────────────────────────────────────────────┘
```

## 🎯 Fluxo de Uso Visual

### 1. Adicionar Livro
```
[Clique "Add Book"] → [Seletor de Arquivo] → [Processamento] → [Livro na Biblioteca]

┌─────────────┐    ┌─────────────┐    ┌─────────────┐    ┌─────────────┐
│ + Add Book  │ →  │ 📁 Selecione│ →  │ ⟳ Loading   │ →  │ 📖 Livro    │
│             │    │ arquivo.epub│    │ Processing  │    │ Adicionado  │
└─────────────┘    └─────────────┘    └─────────────┘    └─────────────┘
```

### 2. Traduzir Livro
```
[Selecionar Idioma] → [Hover no Livro] → [Clique "Translate"] → [Aguardar] → [Concluído]

┌─────────────┐    ┌─────────────┐    ┌─────────────┐    ┌─────────────┐    ┌─────────────┐
│ Translate   │ →  │ 📖 Hover    │ →  │ Translate   │ →  │ 🟡 65%      │ →  │ 🟢 Complete │
│ to: Português│    │ [Translate] │    │ Progress    │    │ Translating │    │ Translated  │
└─────────────┘    └─────────────┘    └─────────────┘    └─────────────┘    └─────────────┘
```

### 3. Ler Livro
```
[Hover no Livro] → [Clique "Read"] → [Navegador Abre] → [Leitura]

┌─────────────┐    ┌─────────────┐    ┌─────────────┐    ┌─────────────┐
│ 📖 Hover    │ →  │ 🌐 Browser  │ →  │ 📄 Capa     │ →  │ 📖 Capítulo │
│ [Read]      │    │ Opens       │    │ & Contents  │    │ Content     │
└─────────────┘    └─────────────┘    └─────────────┘    └─────────────┘
```

## 🎨 Estados Visuais dos Livros

### Status de Tradução
```
🔘 NOT STARTED     🟡 IN PROGRESS     🟢 COMPLETED     🔴 FAILED
┌─────────────┐    ┌─────────────┐    ┌─────────────┐    ┌─────────────┐
│ 📖 [CAPA]   │    │ 📖 [CAPA]   │    │ 📖 [CAPA]   │    │ 📖 [CAPA]   │
│🔘 NOT STARTED│    │🟡 IN PROGRESS│    │🟢 COMPLETED │    │🔴 FAILED    │
│ Título      │    │ Título      │    │ Título      │    │ Título      │
│ Autor       │    │ Autor       │    │ Autor       │    │ Autor       │
│ Idioma      │    │ Idioma      │    │ Idioma      │    │ Idioma      │
└─────────────┘    └─────────────┘    └─────────────┘    └─────────────┘
```

### Ações Disponíveis por Status
```
NOT STARTED:           IN PROGRESS:           COMPLETED:             FAILED:
┌─────────────┐        ┌─────────────┐        ┌─────────────┐        ┌─────────────┐
│ [Read]      │        │ [Read]      │        │ [Read]      │        │ [Read]      │
│ [Translate] │        │ [Cancel]    │        │ [Re-translate]│       │ [Retry]     │
└─────────────┘        └─────────────┘        └─────────────┘        └─────────────┘
```

## 📱 Responsividade

### Desktop (Tela Grande)
```
┌─────────────────────────────────────────────────────────────────────────────┐
│ Header com Logo, Seletor de Idioma e Botão Add Book                        │
├─────────────────────────────────────────────────────────────────────────────┤
│ ┌───┐ ┌───┐ ┌───┐ ┌───┐ ┌───┐  ← 5 livros por linha                       │
│ │ 📖│ │ 📖│ │ 📖│ │ 📖│ │ 📖│                                             │
│ └───┘ └───┘ └───┘ └───┘ └───┘                                             │
│ ┌───┐ ┌───┐ ┌───┐ ┌───┐ ┌───┐                                             │
│ │ 📖│ │ 📖│ │ 📖│ │ 📖│ │ 📖│                                             │
│ └───┘ └───┘ └───┘ └───┘ └───┘                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Tablet (Tela Média)
```
┌─────────────────────────────────────────────────────┐
│ Header empilhado                                    │
│ Logo                                                │
│ Seletor de Idioma                                   │
│ Botão Add Book                                      │
├─────────────────────────────────────────────────────┤
│ ┌─────┐ ┌─────┐ ┌─────┐  ← 3 livros por linha      │
│ │ 📖  │ │ 📖  │ │ 📖  │                            │
│ └─────┘ └─────┘ └─────┘                            │
│ ┌─────┐ ┌─────┐ ┌─────┐                            │
│ │ 📖  │ │ 📖  │ │ 📖  │                            │
│ └─────┘ └─────┘ └─────┘                            │
└─────────────────────────────────────────────────────┘
```

### Mobile (Tela Pequena)
```
┌─────────────────────────────┐
│ Header Vertical             │
│ 📚 ePub Reader             │
│ [Translate to: PT ▼]       │
│ [+ Add Book]               │
├─────────────────────────────┤
│ ┌─────────┐ ┌─────────┐    │ ← 2 livros por linha
│ │   📖    │ │   📖    │    │
│ │ [CAPA]  │ │ [CAPA]  │    │
│ │ Título  │ │ Título  │    │
│ └─────────┘ └─────────┘    │
│ ┌─────────┐ ┌─────────┐    │
│ │   📖    │ │   📖    │    │
│ │ [CAPA]  │ │ [CAPA]  │    │
│ │ Título  │ │ Título  │    │
│ └─────────┘ └─────────┘    │
└─────────────────────────────┘
```

## 🎯 Indicadores Visuais

### Barra de Progresso (Tradução)
```
0%   ░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░
25%  ████████████████████░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░
50%  ████████████████████████████████████████░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░
75%  ████████████████████████████████████████████████████████░░░░░░░░░░░░░░░░░░░░
100% ████████████████████████████████████████████████████████████████████████████
```

### Barra de Progresso (Leitura)
```
Início do capítulo:  ████░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░
Meio do capítulo:    ████████████████████████████████████░░░░░░░░░░░░░░░░░░░░░░░░░░
Final do capítulo:   ████████████████████████████████████████████████████████████
```

### Spinner de Loading
```
Frame 1: ⠋ Loading...
Frame 2: ⠙ Loading...
Frame 3: ⠹ Loading...
Frame 4: ⠸ Loading...
Frame 5: ⠼ Loading...
Frame 6: ⠴ Loading...
Frame 7: ⠦ Loading...
Frame 8: ⠧ Loading...
```

---

## 🎨 Paleta de Cores

### Tema Escuro (Padrão)
```
┌─────────────────────────────────────┐
│ Background: #1a1a1a (Preto suave)  │
│ Cards: #2d2d2d (Cinza escuro)      │
│ Text: #ffffff (Branco)              │
│ Secondary: #b0b0b0 (Cinza claro)   │
│ Primary: #3498db (Azul)             │
│ Success: #27ae60 (Verde)            │
│ Warning: #f39c12 (Amarelo)          │
│ Error: #e74c3c (Vermelho)           │
│ Info: #95a5a6 (Cinza)               │
└─────────────────────────────────────┘
```

### Tema Claro (Leitor)
```
┌─────────────────────────────────────┐
│ Background: #f8f9fa (Branco suave) │
│ Cards: #ffffff (Branco)             │
│ Text: #333333 (Preto suave)         │
│ Secondary: #7f8c8d (Cinza médio)    │
│ Primary: #2980b9 (Azul escuro)      │
│ Borders: #ecf0f1 (Cinza muito claro)│
└─────────────────────────────────────┘
```

---

**🎨 Este guia visual ajuda a entender a interface e fluxos da aplicação de forma intuitiva!**