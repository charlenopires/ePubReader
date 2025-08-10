# ⚡ Início Rápido - ePub Reader Library

## 🚀 Em 5 Minutos

### 1. Instalação Automática
```bash
./setup.sh
```

### 2. Executar Aplicação
```bash
cargo tauri dev
```

### 3. Adicionar Primeiro Livro
- Clique em **"Add Book"**
- Selecione um arquivo `.epub`
- Aguarde o processamento

### 4. Traduzir (Opcional)
- Selecione idioma no header: **"Translate to:"**
- Clique em **"Translate"** no cartão do livro
- Aguarde conclusão (status verde)

### 5. Ler
- Clique em **"Read"** no cartão do livro
- Use setas ← → para navegar
- Ajuste fonte com **A-** / **A+**

---

## 🔧 Solução Rápida de Problemas

### Ollama não está rodando?
```bash
ollama serve
ollama pull llama3.1:8b
```

### Livro não aparece?
- Verifique se é arquivo `.epub` válido
- Aguarde processamento completo

### Tradução muito lenta?
```bash
ollama pull llama2:7b  # Modelo mais rápido
```

### Erro ao abrir livro?
- Verifique se tem navegador padrão configurado
- Tente clicar em "Read" novamente

---

## 📱 Interface Rápida

```
┌─────────────────────────────────────────────────────────────┐
│ 📚 ePub Reader    [Translate to: Português ▼]  [+ Add Book] │
├─────────────────────────────────────────────────────────────┤
│  ┌─────┐  ┌─────┐  ┌─────┐                                  │
│  │📖   │  │📖   │  │📖   │                                  │
│  │Book │  │Book │  │Book │                                  │
│  │ 1   │  │ 2   │  │ 3   │                                  │
│  │🟢   │  │🟡   │  │🔘   │  ← Status: Verde=Traduzido      │
│  └─────┘  └─────┘  └─────┘     Amarelo=Traduzindo          │
│                                 Cinza=Não traduzido         │
└─────────────────────────────────────────────────────────────┘
```

**Hover nos livros mostra:**
- **"Read"** - Ler o livro
- **"Translate"** - Traduzir (se Ollama ativo)

---

## 🌍 Idiomas Disponíveis

| Código | Idioma | Código | Idioma |
|--------|--------|--------|--------|
| `pt` | Português | `en` | English |
| `es` | Español | `fr` | Français |
| `de` | Deutsch | `it` | Italiano |
| `ja` | 日本語 | `ko` | 한국어 |
| `zh` | 中文 | `ru` | Русский |
| `ar` | العربية | `hi` | हिन्दी |

---

## 📁 Onde Ficam os Dados

```
~/.epubreader/ebooks/
├── library.db           # Banco de dados
├── [Nome do Livro]/
│   ├── images/         # Capas e imagens
│   └── html/           # Versão para leitura
```

---

## ⌨️ Atalhos de Leitura

- **← →** - Navegar capítulos
- **Home/Esc** - Voltar ao índice
- **A- / A+** - Ajustar fonte
- **🌓** - Alternar tema
- **⛶** - Tela cheia

---

## 📞 Precisa de Ajuda?

👉 **Consulte o [GUIA_DE_USO.md](GUIA_DE_USO.md) completo**

**Problemas comuns:**
- Ollama não roda → `ollama serve`
- Tradução lenta → Use modelo menor
- Livro não abre → Verifique navegador padrão

---

**🎉 Pronto! Sua biblioteca digital inteligente está funcionando!**