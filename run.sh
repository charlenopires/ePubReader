#!/bin/bash

echo "🚀 Iniciando ePubReader..."
echo "📍 Diretório: $(pwd)"
echo "🔧 Executável: ./src-tauri/target/release/epubreader"
echo ""

# Verificar se o executável existe
if [ ! -f "./src-tauri/target/release/epubreader" ]; then
    echo "❌ Executável não encontrado!"
    echo "📋 Construindo aplicativo..."
    cd src-tauri && cargo build --release && cd ..
fi

echo "▶️  Executando aplicativo..."
./src-tauri/target/release/epubreader

echo "✅ Aplicativo finalizado."