#!/bin/bash
# Script para gerar erros e testar o sistema de alertas
# Gera requisições MCP com erros por ~2 minutos

set -e

# Configuração
USER_ID="e4f1c4f9-e421-40a1-bf41-cc53efc3b7f6"
SERVER="deepwiki"
BASE_URL="http://localhost:8080/mcp/${USER_ID}/${SERVER}"

echo "🔴 Gerando erros para testar alertas..."
echo "   URL: ${BASE_URL}"
echo "   Duração: ~2 minutos"
echo ""

# Função para enviar requisição com erro (session_id inválido)
send_error_request() {
    curl -s -X POST "${BASE_URL}" \
        -H "Content-Type: application/json" \
        -d '{
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/list"
        }' > /dev/null 2>&1 || true
}

# Função para enviar requisição com tool inexistente
send_bad_tool_request() {
    curl -s -X POST "${BASE_URL}" \
        -H "Content-Type: application/json" \
        -d '{
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/call",
            "params": {
                "name": "nonexistent_tool",
                "arguments": {}
            }
        }' > /dev/null 2>&1 || true
}

# Loop por 2 minutos, enviando erro a cada 2 segundos
START_TIME=$(date +%s)
END_TIME=$((START_TIME + 120))
COUNT=0

while [ $(date +%s) -lt $END_TIME ]; do
    # Alterna entre tipos de erro
    if [ $((COUNT % 2)) -eq 0 ]; then
        send_error_request
    else
        send_bad_tool_request
    fi
    
    COUNT=$((COUNT + 1))
    ELAPSED=$(($(date +%s) - START_TIME))
    
    echo -ne "\r⏱️  Tempo: ${ELAPSED}s | Erros enviados: ${COUNT}"
    
    sleep 2
done

echo ""
echo ""
echo "✅ Concluído! ${COUNT} erros gerados em 2 minutos."
echo ""
echo "📊 Agora verifique:"
echo "   1. Dashboard - Error Rate deve estar > 0%"
echo "   2. Se regra de alerta foi criada com threshold menor que o error rate atual,"
echo "      aguarde ~60s para o banner aparecer no Dashboard"
