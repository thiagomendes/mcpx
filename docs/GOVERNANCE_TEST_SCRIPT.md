# Tool Governance - Roteiro de Validação

Este roteiro alterna entre **Terminal (curl+jq)** e **Interface Web** para validar todos os casos de uso.

**Servidor de teste:** Cloudflare (`search_cloudflare_documentation`, `migrate_pages_to_workers_guide`)

---

## Setup Inicial

### Terminal: Criar funções helper

```bash
# Cole isso no terminal uma vez
USER_ID="2d150658-73c1-418d-af2f-a2fbbfbe728b"
SERVER="cloudflare"
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
  
  TOOLS_JSON=$(curl -s -X POST "$BASE/mcp/$USER_ID/$SERVER" \
    -H "Content-Type: application/json" \
    -H "Accept: application/json, text/event-stream" \
    -H "Mcp-Session-Id: $SESSION" \
    -d '{"jsonrpc":"2.0","method":"tools/list","id":2}' \
    | grep "^data:" | sed 's/data: //' | head -1)
  
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
  
  if ! [[ "$CHOICE" =~ ^[0-9]+$ ]] || [ "$CHOICE" -lt 1 ] || [ "$CHOICE" -gt ${#TOOLS[@]} ]; then
    echo "❌ Escolha inválida"
    return 1
  fi
  
  SELECTED_TOOL="${TOOLS[$((CHOICE-1))]}"
  echo ""
  echo "🎯 Tool selecionada: $SELECTED_TOOL"
  echo ""
  
  # Pedir argumentos baseado na tool
  if [[ "$SELECTED_TOOL" == *"search"* ]] || [[ "$SELECTED_TOOL" == *"documentation"* ]]; then
    read -p "Query (ex: How to use Workers?): " QUERY
    ARGS="{\"query\":\"$QUERY\"}"
  else
    read -p "Argumentos JSON (ou enter para {}): " ARGS
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
    | grep "^data:" | sed 's/data: //' | jq '.' 2>/dev/null | head -50
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
search_cloudflare_documentation
migrate_pages_to_workers_guide
```

### 1.2 Interface: Verificar estado
1. Acesse: http://localhost:3000/servers/cloudflare
2. Clique em **"Test Connection"** (para carregar lista de tools)
3. Role até **"Tool Governance"**
4. Verifique: Toggle "Limit tool exposure" desligado

---

## Caso 2: Allowlist (Permitir apenas 1 tool)

### 2.1 Interface: Configurar allowlist
1. Ative o toggle **"Limit tool exposure"**
2. Selecione **"Allow only selected"**
3. Clique na tool **"search_cloudflare_documentation"** (chip verde)
4. Clique em **"Save Rules"**

### 2.2 Terminal: Verificar filtro aplicado
```bash
tools_list
```

**Resultado esperado:**
```
search_cloudflare_documentation
```
(Apenas 1 tool! A outra foi filtrada)

---

## Caso 3: Mudar para Blocklist

### 3.1 Interface: Trocar para blocklist
1. Selecione **"Block selected"**
2. A lista será limpa
3. Clique na tool **"migrate_pages_to_workers_guide"** (chip vermelho)
4. Clique em **"Save Rules"**

### 3.2 Terminal: Verificar blocklist
```bash
tools_list
```

**Resultado esperado:**
```
search_cloudflare_documentation
```
(1 tool - `migrate_pages_to_workers_guide` foi bloqueada)

---

## Caso 4: Adicionar Prefix Global

### 4.1 Interface: Configurar prefix
1. Desative toggle "Limit tool exposure"
2. No campo **"Tool Prefix"**, digite: `cf`
3. Clique em **"Save Rules"**

### 4.2 Terminal: Verificar prefix
```bash
tools_list
```

**Resultado esperado:**
```
cf_search_cloudflare_documentation
cf_migrate_pages_to_workers_guide
```
(Ambas tools com prefix `cf_`)

---

## Caso 5: Prefix + Allowlist (combinado)

### 5.1 Interface: Combinar prefix com allowlist
1. Mantenha o prefix `cf`
2. Ative toggle e selecione **"Allow only selected"**
3. Clique na tool **"search_cloudflare_documentation"**
4. Clique em **"Save Rules"**

### 5.2 Terminal: Verificar combinação
```bash
tools_list
```

**Resultado esperado:**
```
cf_search_cloudflare_documentation
```
(Apenas 1 tool, com prefix)

---

## Caso 6: Limpar Tudo

### 6.1 Interface: Limpar regras
1. Clique em **"Clear Rules"**
2. Confirme no popup

### 6.2 Terminal: Verificar estado original
```bash
tools_list
```

**Resultado esperado:**
```
search_cloudflare_documentation
migrate_pages_to_workers_guide
```
(Volta ao estado original)

---

## Caso 7: Testar tools/call com Allowlist

### 7.1 Interface: Configurar allowlist restritiva
1. Ative toggle e selecione **"Allow only selected"**
2. Selecione apenas **"search_cloudflare_documentation"**
3. Clique em **"Save Rules"**

### 7.2 Terminal: Chamar tool PERMITIDA
```bash
mcp_test
# Escolha 1 (search_cloudflare_documentation)
# Query: How to use Workers?
```

**Resultado esperado:** Resposta do Cloudflare sobre Workers

### 7.3 Terminal: Tentar chamar tool BLOQUEADA
```bash
SESSION=$(curl -s -i -X POST "$BASE/mcp/$USER_ID/$SERVER" \
  -H "Content-Type: application/json" \
  -H "Accept: application/json, text/event-stream" \
  -d '{"jsonrpc":"2.0","method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"1.0"}},"id":1}' \
  | grep "^mcp-session-id:" | cut -d' ' -f2 | tr -d '\r\n')

curl -s -X POST "$BASE/mcp/$USER_ID/$SERVER" \
  -H "Content-Type: application/json" \
  -H "Accept: application/json, text/event-stream" \
  -H "Mcp-Session-Id: $SESSION" \
  -d '{"jsonrpc":"2.0","method":"tools/call","params":{"name":"migrate_pages_to_workers_guide","arguments":{}},"id":3}'
```

**Resultado esperado:** Erro 403 Forbidden
```
Tool 'migrate_pages_to_workers_guide' is not allowed by governance policy
```

---

## Caso 8: Testar strip de prefix no tools/call

### 8.1 Interface: Configurar prefix
1. Desative toggle (All Tools)
2. No campo "Tool Prefix", digite: `cf`
3. Clique em "Save Rules"

### 8.2 Terminal: Verificar que tools têm prefix
```bash
tools_list
```

**Resultado esperado:**
```
cf_search_cloudflare_documentation
cf_migrate_pages_to_workers_guide
```

### 8.3 Terminal: Chamar tool COM prefix
```bash
mcp_test
# Escolha 1 (cf_search_cloudflare_documentation)
# Query: How to use Workers?
```

**Resultado esperado:** Resposta do Cloudflare (mcpx removeu o prefix `cf_` antes de enviar)

---

## Resumo dos Casos

| Caso | Config | Tools Esperadas |
|------|--------|-----------------|
| 1 | Nenhuma | 2 tools (original) |
| 2 | Allowlist: search | 1 tool |
| 3 | Blocklist: migrate | 1 tool |
| 4 | Prefix: cf | 2 tools (com prefix) |
| 5 | Prefix + Allowlist | 1 tool (com prefix) |
| 6 | Clear | 2 tools (original) |
| 7 | Allowlist + tools/call | Permitir/Bloquear call |
| 8 | Prefix + tools/call | Strip prefix |
