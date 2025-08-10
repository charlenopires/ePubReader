# ❓ Perguntas Frequentes (FAQ) - ePub Reader Library

## 📚 Sobre a Aplicação

### **Q: O que é o ePub Reader Library?**
**A:** É uma aplicação desktop moderna para gerenciar e ler livros eletrônicos EPUB, com capacidade de tradução automática usando inteligência artificial (Ollama).

### **Q: Quais formatos de livro são suportados?**
**A:** Atualmente apenas EPUB. Suporte para PDF e MOBI está planejado para futuras versões.

### **Q: A aplicação é gratuita?**
**A:** Sim, é completamente gratuita e open source.

### **Q: Funciona offline?**
**A:** Parcialmente. A leitura funciona offline, mas a tradução requer Ollama rodando localmente (que funciona offline após baixar os modelos).

---

## 🚀 Instalação e Configuração

### **Q: Quais são os requisitos do sistema?**
**A:** 
- **SO:** Windows 10+, macOS 10.15+, ou Linux moderno
- **RAM:** Mínimo 4GB, recomendado 8GB+ (para tradução)
- **Espaço:** 2GB+ livres (modelos de IA ocupam espaço)
- **Rust:** Versão 1.70+
- **Node.js:** Versão 16+

### **Q: Como instalo o Ollama?**
**A:** 
```bash
# macOS/Linux
curl -fsSL https://ollama.ai/install.sh | sh

# Windows
# Baixe de https://ollama.ai/download
```

### **Q: Qual modelo do Ollama devo usar?**
**A:** Recomendamos `llama3.1:8b` para melhor qualidade/velocidade. Para máquinas mais lentas, use `llama2:7b`.

### **Q: A instalação falhou, o que fazer?**
**A:** 
1. Verifique se Rust e Node.js estão instalados
2. Execute `./setup.sh` novamente
3. Verifique os logs de erro
4. Consulte a seção "Solução de Problemas" no guia

---

## 📖 Uso da Aplicação

### **Q: Como adiciono livros?**
**A:** Clique em "Add Book" no canto superior direito e selecione um arquivo `.epub` do seu computador.

### **Q: Onde ficam armazenados meus livros?**
**A:** Em `~/.epubreader/ebooks/` (Linux/macOS) ou `%APPDATA%\.epubreader\ebooks\` (Windows).

### **Q: Posso organizar os livros em pastas?**
**A:** Atualmente não há sistema de pastas/coleções, mas está planejado para futuras versões.

### **Q: Como removo um livro da biblioteca?**
**A:** Atualmente não há interface para remoção. Você pode deletar manualmente a pasta do livro em `~/.epubreader/ebooks/`.

---

## 🌍 Tradução

### **Q: Quais idiomas são suportados para tradução?**
**A:** 12 idiomas: Português, Inglês, Espanhol, Francês, Alemão, Italiano, Japonês, Coreano, Chinês, Russo, Árabe e Hindi.

### **Q: Quanto tempo demora uma tradução?**
**A:** Varia conforme o tamanho do livro e modelo usado:
- Livro pequeno (100 páginas): 5-15 minutos
- Livro médio (300 páginas): 15-45 minutos  
- Livro grande (500+ páginas): 45+ minutos

### **Q: A tradução é boa?**
**A:** Depende do modelo usado. `llama3.1:8b` oferece qualidade muito boa, especialmente para idiomas populares como português, inglês e espanhol.

### **Q: Posso traduzir o mesmo livro para vários idiomas?**
**A:** Atualmente não. Cada livro pode ter apenas uma tradução ativa. Suporte para múltiplas traduções está planejado.

### **Q: A tradução falhou, o que fazer?**
**A:** 
1. Verifique se Ollama está rodando (`ollama serve`)
2. Verifique se o modelo está instalado (`ollama list`)
3. Tente novamente com um modelo menor
4. Verifique se há espaço em disco suficiente

---

## 📚 Leitura

### **Q: Como leio um livro?**
**A:** Passe o mouse sobre o cartão do livro e clique em "Read". O livro abrirá no seu navegador padrão.

### **Q: Posso ler dentro da própria aplicação?**
**A:** Atualmente não. O leitor integrado está planejado para futuras versões.

### **Q: Como navego entre capítulos?**
**A:** Use as setas do teclado (← →), botões Previous/Next, ou volte ao índice e selecione o capítulo.

### **Q: Posso ajustar o tamanho da fonte?**
**A:** Sim! Use os botões A- e A+ no canto superior direito durante a leitura.

### **Q: Há modo escuro?**
**A:** Sim! Clique no botão 🌓 durante a leitura para alternar entre temas claro e escuro.

### **Q: Minha posição de leitura é salva?**
**A:** Sim, automaticamente. Quando reabrir o livro, voltará onde parou.

---

## 🔧 Problemas Técnicos

### **Q: "Ollama is not running" - o que fazer?**
**A:** 
1. Abra um terminal e execute `ollama serve`
2. Aguarde alguns segundos
3. Clique em "Check Again" na aplicação

### **Q: O livro não aparece após adicionar**
**A:** 
1. Verifique se é um arquivo EPUB válido
2. Aguarde o processamento completo (pode demorar)
3. Verifique se há espaço em disco
4. Reinicie a aplicação

### **Q: A tradução está muito lenta**
**A:** 
1. Use um modelo menor: `ollama pull llama2:7b`
2. Feche outros programas pesados
3. Verifique se tem RAM suficiente (8GB+ recomendado)

### **Q: Erro ao abrir livro para leitura**
**A:** 
1. Verifique se tem um navegador padrão configurado
2. Tente clicar em "Read" novamente
3. Verifique se os arquivos HTML foram gerados

### **Q: A aplicação não inicia**
**A:** 
1. Verifique se Rust está instalado: `cargo --version`
2. Verifique se Node.js está instalado: `node --version`
3. Execute `cargo tauri dev` no terminal para ver erros
4. Reinstale as dependências

---

## 💾 Dados e Backup

### **Q: Como faço backup dos meus livros?**
**A:** 
```bash
# Backup completo
tar -czf biblioteca-backup.tar.gz ~/.epubreader/

# Restaurar backup
tar -xzf biblioteca-backup.tar.gz -C ~/
```

### **Q: Posso sincronizar entre dispositivos?**
**A:** Atualmente não há sincronização automática. Você pode copiar manualmente a pasta `~/.epubreader/` entre dispositivos.

### **Q: Quanto espaço ocupam os livros?**
**A:** 
- Livro original: tamanho do arquivo EPUB
- Imagens extraídas: 1-10MB por livro
- Tradução: 2-5x o tamanho do texto original
- HTML gerado: 1-5MB por livro

### **Q: Posso deletar traduções antigas?**
**A:** Atualmente não há interface para isso. Você pode deletar manualmente no banco SQLite ou aguardar futuras versões.

---

## 🔄 Atualizações e Desenvolvimento

### **Q: Como atualizo a aplicação?**
**A:** 
1. Faça backup dos dados
2. Baixe a nova versão
3. Execute `cargo tauri build`
4. Substitua o executável

### **Q: Como reporto bugs?**
**A:** Abra uma issue no repositório GitHub com:
- Descrição do problema
- Passos para reproduzir
- Logs de erro
- Informações do sistema

### **Q: Posso contribuir com o projeto?**
**A:** Sim! O projeto é open source. Faça um fork, implemente melhorias e abra um Pull Request.

### **Q: Quais recursos estão planejados?**
**A:** 
- [ ] Suporte para PDF e MOBI
- [ ] Leitor integrado na aplicação
- [ ] Sistema de coleções/pastas
- [ ] Múltiplas traduções por livro
- [ ] Sincronização em nuvem
- [ ] Anotações e marcadores
- [ ] Estatísticas de leitura

---

## 🎯 Dicas e Truques

### **Q: Como otimizo a performance?**
**A:** 
1. Use modelos menores para tradução mais rápida
2. Feche outros programas durante tradução
3. Traduza apenas livros que vai ler
4. Mantenha espaço livre em disco

### **Q: Qual é o melhor fluxo de trabalho?**
**A:** 
1. Adicione vários livros de uma vez
2. Selecione idioma de preferência
3. Traduza em ordem de prioridade de leitura
4. Use modo tela cheia para leitura imersiva

### **Q: Como melhoro a qualidade da tradução?**
**A:** 
1. Use modelos maiores (`llama3.1:8b` ou `llama3.1:13b`)
2. Certifique-se que o idioma original está correto
3. Para idiomas menos comuns, considere traduzir primeiro para inglês

### **Q: Posso usar em máquinas mais antigas?**
**A:** 
Sim, mas com limitações:
- Use modelos menores (`llama2:7b`)
- Traduza livros menores primeiro
- Considere aumentar RAM se possível
- Feche outros programas durante uso

---

## 🆘 Suporte

### **Q: Onde encontro mais ajuda?**
**A:** 
1. **Guia Completo:** `GUIA_DE_USO.md`
2. **Início Rápido:** `INICIO_RAPIDO.md`
3. **Guia Visual:** `GUIA_VISUAL.md`
4. **Issues GitHub:** Para bugs e sugestões
5. **Documentação Ollama:** https://ollama.ai/

### **Q: A aplicação é segura?**
**A:** 
Sim, a aplicação:
- Não envia dados para internet (exceto Ollama local)
- Armazena tudo localmente
- É open source (código auditável)
- Não coleta dados pessoais

### **Q: Funciona em servidores/headless?**
**A:** Não, é uma aplicação desktop com interface gráfica. Para uso em servidor, seria necessário adaptação.

---

**💡 Não encontrou sua pergunta? Consulte os outros guias ou abra uma issue no repositório!**