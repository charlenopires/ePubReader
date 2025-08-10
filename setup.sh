#!/bin/bash

echo "🚀 Setting up ePub Reader Library..."

# Check if Rust is installed
if ! command -v cargo &> /dev/null; then
    echo "❌ Rust is not installed. Please install Rust first:"
    echo "   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    exit 1
fi

# Check if Node.js is installed
if ! command -v node &> /dev/null; then
    echo "❌ Node.js is not installed. Please install Node.js first:"
    echo "   https://nodejs.org/"
    exit 1
fi

echo "✅ Rust and Node.js are installed"

# Install Tauri CLI
echo "📦 Installing Tauri CLI..."
cargo install tauri-cli --locked

# Check if Ollama is installed
if ! command -v ollama &> /dev/null; then
    echo "⚠️  Ollama is not installed. Installing Ollama..."
    
    # Detect OS and install Ollama
    if [[ "$OSTYPE" == "darwin"* ]]; then
        # macOS
        curl -fsSL https://ollama.ai/install.sh | sh
    elif [[ "$OSTYPE" == "linux-gnu"* ]]; then
        # Linux
        curl -fsSL https://ollama.ai/install.sh | sh
    else
        echo "❌ Please install Ollama manually from https://ollama.ai/"
        echo "   Then run: ollama serve"
        echo "   And: ollama pull llama3.1:8b"
    fi
else
    echo "✅ Ollama is already installed"
fi

# Start Ollama service (if not running)
if ! pgrep -x "ollama" > /dev/null; then
    echo "🔄 Starting Ollama service..."
    ollama serve &
    sleep 3
fi

# Pull recommended model
echo "📥 Pulling recommended translation model..."
ollama pull llama3.1:8b

echo ""
echo "🎉 Setup complete!"
echo ""
echo "To start the application:"
echo "  cargo tauri dev"
echo ""
echo "To build for production:"
echo "  cargo tauri build"
echo ""
echo "Make sure Ollama is running before using translation features:"
echo "  ollama serve"
echo ""