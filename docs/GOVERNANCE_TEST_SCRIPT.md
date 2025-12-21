# Tool Governance - Roteiro de Validação

Este roteiro alterna entre **Terminal (curl+jq)** e **Interface Web** para validar todos os casos de uso.

---

## Setup Inicial

### Terminal: Criar função helper

```bash
# Cole isso no terminal uma vez
USER_ID="2d150658-73c1-418d-af2f-a2fbbfbe728b"
SERVER="deepwiki"
BASE="http://localhost:8080"

# Função interativa: lista tools e permite escolher qual consumir
mcp_test() {
  echo "🔄 Iniciando sessão MCP..."
  
  SESSION=$(curl -s -i -X POST "$BASE/mcp/$USER_ID/$SERVER" \
    -H "Content-Type: application/json" \
    -H "Accept: application/json, text/event-stream" \
    -d '{"jsonrpc":"2.0","method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"1.0"}},"id":1}' \
    | grep "^mcp-session-id:" | cut -d' ' -f2 | tr -d '\r\n')
  
  if [ -z "$SESSION" ]; then
    echo "❌ Erro: Não foi possível obter Session ID"
    return 1
  fi
  
  echo "✅ Session: $SESSION"
  echo ""
  echo "📋 Buscando tools disponíveis..."
  
  # Buscar lista de tools
  TOOLS_JSON=$(curl -s -X POST "$BASE/mcp/$USER_ID/$SERVER" \
    -H "Content-Type: application/json" \
    -H "Accept: application/json, text/event-stream" \
    -H "Mcp-Session-Id: $SESSION" \
    -d '{"jsonrpc":"2.0","method":"tools/list","id":2}' \
    | grep "^data:" | sed 's/data: //' | head -1)
  
  # Extrair nomes das tools
  TOOLS=($(echo "$TOOLS_JSON" | jq -r '.result.tools[].name' 2>/dev/null))
  
  if [ ${#TOOLS[@]} -eq 0 ]; then
    echo "❌ Nenhuma tool encontrada"
    return 1
  fi
  
  echo ""
  echo "🔧 Tools disponíveis:"
  echo "   0) Sair (não consumir nenhuma)"
  for i in "${!TOOLS[@]}"; do
    echo "   $((i+1))) ${TOOLS[$i]}"
  done
  
  echo ""
  read -p "Escolha uma tool (0-${#TOOLS[@]}): " CHOICE
  
  if [ "$CHOICE" = "0" ] || [ -z "$CHOICE" ]; then
    echo "👋 Saindo..."
    return 0
  fi
  
  # Validar escolha
  if ! [[ "$CHOICE" =~ ^[0-9]+$ ]] || [ "$CHOICE" -lt 1 ] || [ "$CHOICE" -gt ${#TOOLS[@]} ]; then
    echo "❌ Escolha inválida"
    return 1
  fi
  
  SELECTED_TOOL="${TOOLS[$((CHOICE-1))]}"
  echo ""
  echo "🎯 Tool selecionada: $SELECTED_TOOL"
  echo ""
  
  # Pedir argumentos baseado na tool
  if [[ "$SELECTED_TOOL" == *"ask_question"* ]]; then
    read -p "Repo (ex: facebook/react): " REPO
    read -p "Pergunta: " QUESTION
    ARGS="{\"repoName\":\"$REPO\",\"question\":\"$QUESTION\"}"
  elif [[ "$SELECTED_TOOL" == *"read_wiki"* ]]; then
    read -p "Repo (ex: facebook/react): " REPO
    ARGS="{\"repoName\":\"$REPO\"}"
  else
    read -p "Argumentos JSON: " ARGS
    ARGS="${ARGS:-{}}"
  fi
  
  echo ""
  echo "🚀 Chamando tool..."
  echo ""
  
  curl -s -X POST "$BASE/mcp/$USER_ID/$SERVER" \
    -H "Content-Type: application/json" \
    -H "Accept: application/json, text/event-stream" \
    -H "Mcp-Session-Id: $SESSION" \
    -d "{\"jsonrpc\":\"2.0\",\"method\":\"tools/call\",\"params\":{\"name\":\"$SELECTED_TOOL\",\"arguments\":$ARGS},\"id\":3}" \
    | grep "^data:" | sed 's/data: //' | jq '.' 2>/dev/null || echo "(resposta raw)"
}

# Função simples: só lista tools
tools_list() {
  SESSION=$(curl -s -i -X POST "$BASE/mcp/$USER_ID/$SERVER" \
    -H "Content-Type: application/json" \
    -H "Accept: application/json, text/event-stream" \
    -d '{"jsonrpc":"2.0","method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"1.0"}},"id":1}' \
    | grep "^mcp-session-id:" | cut -d' ' -f2 | tr -d '\r\n')
  
  echo "Tools disponíveis:"
  curl -s -X POST "$BASE/mcp/$USER_ID/$SERVER" \
    -H "Content-Type: application/json" \
    -H "Accept: application/json, text/event-stream" \
    -H "Mcp-Session-Id: $SESSION" \
    -d '{"jsonrpc":"2.0","method":"tools/list","id":2}' \
    | grep "^data:" | sed 's/data: //' | jq -r '.result.tools[].name' 2>/dev/null
}

echo "✅ Funções carregadas: tools_list, mcp_test"
```

---

## Caso 1: Estado Inicial (Sem Governance)

### 1.1 Terminal: Verificar tools sem filtro
```bash
tools_list
```

**Resultado esperado:**
```
read_wiki_structure
read_wiki_contents
ask_question
```

### 1.2 Interface: Verificar estado
1. Acesse: http://localhost:3000/servers/deepwiki
2. Clique em **"Test Connection"** (para carregar lista de tools)
3. Role até **"Tool Governance"**
4. Verifique: "All Tools" selecionado, sem prefix, sem filtros

---

## Caso 2: Allowlist (Permitir apenas algumas tools)

### 2.1 Interface: Configurar allowlist
1. Em **"Tool Governance"**, clique em **"Allowlist"**
2. Clique na tool **"ask_question"** (chip verde aparece)
3. Clique em **"Save Rules"**
4. Verifique: Mensagem "Governance rules saved!"

### 2.2 Terminal: Verificar filtro aplicado
```bash
tools_list
```

**Resultado esperado:**
```
ask_question
```
(Apenas 1 tool! As outras foram filtradas)

---

## Caso 3: Adicionar segunda tool à Allowlist

### 3.1 Interface: Adicionar mais uma tool
1. Clique na tool **"read_wiki_contents"** (outro chip verde aparece)
2. Clique em **"Save Rules"**

### 3.2 Terminal: Verificar 2 tools
```bash
tools_list
```

**Resultado esperado:**
```
ask_question
read_wiki_contents
```

---

## Caso 4: Mudar para Blocklist

### 4.1 Interface: Trocar para blocklist
1. Clique em **"Blocklist"** (vermelho)
2. A lista de tools selecionadas será limpa
3. Clique na tool **"read_wiki_structure"** (chip vermelho aparece)
4. Clique em **"Save Rules"**

### 4.2 Terminal: Verificar blocklist
```bash
tools_list
```

**Resultado esperado:**
```
read_wiki_contents
ask_question
```
(2 tools - `read_wiki_structure` foi bloqueada)

---

## Caso 5: Adicionar Prefix Global

### 5.1 Interface: Configurar prefix
1. Clique em **"All Tools"** (limpa blocklist)
2. No campo **"Tool Prefix"** (no topo), digite: `wiki`
3. Clique em **"Save Rules"**

### 5.2 Terminal: Verificar prefix
```bash
tools_list
```

**Resultado esperado:**
```
wiki_read_wiki_structure
wiki_read_wiki_contents
wiki_ask_question
```
(Todas as 3 tools com prefix `wiki_`)

---

## Caso 6: Prefix + Allowlist (combinado)

### 6.1 Interface: Combinar prefix com allowlist
1. Mantenha o prefix `wiki`
2. Clique em **"Allowlist"**
3. Clique na tool **"ask_question"**
4. Clique em **"Save Rules"**

### 6.2 Terminal: Verificar combinação
```bash
tools_list
```

**Resultado esperado:**
```
wiki_ask_question
```
(Apenas 1 tool, com prefix)

---

## Caso 7: Limpar Tudo

### 7.1 Interface: Limpar regras
1. Clique em **"Clear Rules"**
2. Confirme no popup

### 7.2 Terminal: Verificar estado original
```bash
tools_list
```

**Resultado esperado:**
```
read_wiki_structure
read_wiki_contents
ask_question
```
(Volta ao estado original, sem prefix, sem filtro)

---

## Caso 8: Testar tools/call com governance

### 8.1 Interface: Configurar allowlist restritiva
1. Clique em **"Allowlist"**
2. Selecione apenas **"ask_question"**
3. Clique em **"Save Rules"**

### 8.2 Terminal: Tentar chamar tool permitida
```bash
SESSION=$(curl -s -i -X POST "http://localhost:8080/mcp/2d150658-73c1-418d-af2f-a2fbbfbe728b/deepwiki" \
  -H "Content-Type: application/json" \
  -H "Accept: application/json, text/event-stream" \
  -d '{"jsonrpc":"2.0","method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"1.0"}},"id":1}' \
  | grep "^mcp-session-id:" | cut -d' ' -f2 | tr -d '\r\n')

# Chamar tool PERMITIDA
curl -s -X POST "http://localhost:8080/mcp/2d150658-73c1-418d-af2f-a2fbbfbe728b/deepwiki" \
  -H "Content-Type: application/json" \
  -H "Accept: application/json, text/event-stream" \
  -H "Mcp-Session-Id: $SESSION" \
  -d '{"jsonrpc":"2.0","method":"tools/call","params":{"name":"ask_question","arguments":{"repoName":"facebook/react","question":"What is React?"}},"id":3}' | head -c 500
```

**Resultado esperado:** Resposta do DeepWiki sobre React

### 8.3 Terminal: Tentar chamar tool BLOQUEADA
```bash
curl -s -X POST "http://localhost:8080/mcp/2d150658-73c1-418d-af2f-a2fbbfbe728b/deepwiki" \
  -H "Content-Type: application/json" \
  -H "Accept: application/json, text/event-stream" \
  -H "Mcp-Session-Id: $SESSION" \
  -d '{"jsonrpc":"2.0","method":"tools/call","params":{"name":"read_wiki_structure","arguments":{"repoName":"facebook/react"}},"id":4}'
```

**Resultado esperado:** Erro 403 Forbidden
```
Tool 'read_wiki_structure' is not allowed by governance policy
```

---

## Resumo dos Casos

| Caso | Config | Tools Esperadas |
|------|--------|-----------------|
| 1 | Nenhuma | 3 tools (original) |
| 2 | Allowlist: ask_question | 1 tool |
| 3 | Allowlist: +read_wiki_contents | 2 tools |
| 4 | Blocklist: read_wiki_structure | 2 tools |
| 5 | Prefix: wiki | 3 tools (com prefix) |
| 6 | Prefix + Allowlist | 1 tool (com prefix) |
| 7 | Clear | 3 tools (original) |
| 8 | Allowlist + tools/call | Permitir/Bloquear tool call |
| 9 | Prefix + tools/call | Strip prefix antes de enviar |

---

## Caso 9: Testar strip de prefix no tools/call

### 9.1 Interface: Configurar prefix
1. Desative "Limit tool exposure" (toggle off)
2. No campo "Tool Prefix", digite: `wiki`
3. Clique em "Save Rules"

### 9.2 Terminal: Verificar que tools têm prefix
```bash
tools_list
```

**Resultado esperado:**
```
wiki_read_wiki_structure
wiki_read_wiki_contents
wiki_ask_question
```

### 9.3 Terminal: Chamar tool COM prefix (como cliente vê)
```bash
SESSION=$(curl -s -i -X POST "http://localhost:8080/mcp/2d150658-73c1-418d-af2f-a2fbbfbe728b/deepwiki" \
  -H "Content-Type: application/json" \
  -H "Accept: application/json, text/event-stream" \
  -d '{"jsonrpc":"2.0","method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"1.0"}},"id":1}' \
  | grep "^mcp-session-id:" | cut -d' ' -f2 | tr -d '\r\n')

# Chamar com o prefix wiki_ (mcpx deve remover antes de enviar para DeepWiki)
curl -s -X POST "http://localhost:8080/mcp/2d150658-73c1-418d-af2f-a2fbbfbe728b/deepwiki" \
  -H "Content-Type: application/json" \
  -H "Accept: application/json, text/event-stream" \
  -H "Mcp-Session-Id: $SESSION" \
  -d '{"jsonrpc":"2.0","method":"tools/call","params":{"name":"wiki_ask_question","arguments":{"repoName":"facebook/react","question":"What is React?"}},"id":3}' | head -c 500
```

**Resultado esperado:** Resposta do DeepWiki (mcpx removeu o prefix `wiki_` antes de enviar)
