#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

DEFAULT_INPUT_DIR="$REPO_ROOT/input"
DEFAULT_OUTPUT_DIR="$REPO_ROOT/output"

########################################
# CONFIGURAÇÃO PRINCIPAL
########################################

# Diretório alvo para escanear
TARGET_DIR="${TARGET_DIR:-$DEFAULT_INPUT_DIR}"

# Diretório de saída para os arquivos gerados
OUTPUT_DIR="${OUTPUT_DIR:-$DEFAULT_OUTPUT_DIR}"

# Sufixo do arquivo de saída
OUTPUT_FILE_SUFFIX="${OUTPUT_FILE_SUFFIX:-_project_code.txt}"

# Tamanho máximo por arquivo (em bytes) - padrão 2MB
MAX_SIZE_BYTES="${MAX_SIZE_BYTES:-2097152}"

########################################
# LISTAS DE EXCLUSÃO
########################################

# Lista de ARQUIVOS a ignorar (separados por |)
# Você pode adicionar mais arquivos aqui ou via variável de ambiente IGNORE_FILES_EXTRA
IGNORE_FILES_BASE='.DS_Store|Thumbs.db|.env|.env.local|.env.production|.env.development|*.key|*.pem|*.p12|*.pfx|*.log|*.pid|*.seed|*.sqlite|*.sqlite3|*.db|desktop.ini|*.swp|*.swo|*~|.~lock.*|._*'
IGNORE_FILES_EXTRA="${IGNORE_FILES_EXTRA:-}"

# Lista de DIRETÓRIOS a ignorar (separados por |)
# Diretórios padrão de build, dependências e cache
IGNORE_DIRS_BASE='node_modules|dist|build|target|bin|.git|.next|coverage|.turbo|.cache|.expo|.gradle|.mvn|.settings|Pods|DerivedData|.idea|.vscode|out|tmp|.parcel-cache|.sass-cache|.nuxt|.svelte-kit|__pycache__|.pytest_cache|cmake-build-debug|cmake-build-release|CMakeFiles|.dart_tool|.pub-cache|.pub|.android|.ios|.macos|.windows|.linux|.metadata|.packages|.flutter-plugins|.flutter-plugins-dependencies|vendor|bower_components|jspm_packages|web_modules|.yarn|.pnp.*'
IGNORE_DIRS_EXTRA="${IGNORE_DIRS_EXTRA:-}"

# Lista de CAMINHOS RELATIVOS de pastas a ignorar
# Útil para ignorar bibliotecas específicas ou subpastas profundas
# Exemplo: "src/vendor/large-library|assets/generated"
IGNORE_PATHS="${IGNORE_PATHS:-}"

# Lista de CAMINHOS ABSOLUTOS de pastas a ignorar
# Útil para ignorar diretórios específicos com caminho completo
# Exemplo: "/home/user/project/vendor/huge-lib|/opt/shared/legacy"
IGNORE_ABSOLUTE_PATHS="${IGNORE_ABSOLUTE_PATHS:-}"

# Combina as listas com extras (se fornecidas)
IGNORE_FILES_PATTERN="${IGNORE_FILES_BASE}${IGNORE_FILES_EXTRA:+|$IGNORE_FILES_EXTRA}"
IGNORE_DIRS_PATTERN="${IGNORE_DIRS_BASE}${IGNORE_DIRS_EXTRA:+|$IGNORE_DIRS_EXTRA}"

########################################
# EXTENSÕES E ARQUIVOS DE CONFIGURAÇÃO
########################################

# Extensões de código a incluir
CODE_EXTS=(
    # C/C++
    "c" "cpp" "cc" "cxx" "c++" "h" "hpp" "hxx" "h++" "hh"
    
    # Objective-C / Swift
    "m" "mm" "swift"
    
    # JavaScript / TypeScript
    "js" "jsx" "mjs" "cjs" "ts" "tsx" "mts" "cts"
    
    # Java / Kotlin
    "java" "kt" "kts"
    
    # Dart / Flutter
    "dart"
    
    # HTML/CSS
    "html" "htm" "css" "scss" "sass" "less"
    
    # Markdown
    "md" "mdx" "markdown"
    
    # Python
    "py" "pyx" "pyi"
    
    # Rust
    "rs"
    
    # Go
    "go"
    
    # Ruby
    "rb" "erb"
    
    # PHP
    "php"
    
    # Shell scripts
    "sh" "bash" "zsh" "fish"
    
    # Metal shaders
    "metal"
)

# Arquivos de configuração importantes
CONFIG_FILES=(
    # Node/TS/Bundlers
    "package.json" "package-lock.json" "pnpm-lock.yaml" "yarn.lock" "bun.lockb"
    "tsconfig.json" "tsconfig.*.json" "jsconfig.json"
    "vite.config.*" "webpack.config.*" "rollup.config.*"
    "babel.config.*" ".babelrc*"
    "next.config.*" ".eslintrc*" ".prettierrc*" "prettier.config.*"
    ".npmrc" ".nvmrc" ".node-version"
    
    # Java/Maven/Gradle
    "pom.xml" "build.gradle*" "settings.gradle*" "gradle.properties"
    "gradlew" "gradlew.bat" "mvnw" "mvnw.cmd"
    ".classpath" ".project"
    
    # Spring Boot
    "application*.properties" "application*.yml" "application*.yaml"
    "bootstrap*.properties" "bootstrap*.yml" "bootstrap*.yaml"
    
    # Front-end
    "postcss.config.*" "tailwind.config.*" "stylelint.config.*" ".stylelintrc*"
    
    # C/C++ Build Files
    "Makefile" "makefile" "GNUmakefile" "CMakeLists.txt" "*.cmake"
    "configure" "configure.ac" "configure.in"
    "*.pro" "*.pri"  # Qt project files
    "meson.build" "meson_options.txt"
    "BUILD" "BUILD.bazel" "WORKSPACE"  # Bazel
    
    # Flutter
    "pubspec.yaml" "pubspec.lock" "pubspec_overrides.yaml"
    ".metadata" ".packages" ".dart_tool/package_config.json"
    "analysis_options.yaml" "*.iml"
    "android/local.properties" "ios/Podfile*"
    
    # Documentation & Others
    "README*" "LICENSE*" ".gitignore" ".gitattributes"
    "Dockerfile" "docker-compose.yml" "docker-compose.yaml"
    ".dockerignore" "Procfile" "requirements.txt" "Gemfile" "Gemfile.lock"
    "Cargo.toml" "Cargo.lock" "go.mod" "go.sum"
    
    # Xcode
    "*.xcodeproj" "*.xcworkspace" "*.xcscheme" "*.pbxproj"
    "Info.plist" "Entitlements.plist"
)

########################################
# FUNÇÕES AUXILIARES
########################################

# Função: obtém tamanho do arquivo de forma portátil (macOS/Linux)
get_size_bytes() {
    local f="$1"
    if size=$(stat -f%z "$f" 2>/dev/null); then
        echo "$size"
    else
        stat -c%s "$f" 2>/dev/null || echo "0"
    fi
}

# Função: formata bytes para exibição legível
format_bytes() {
    local bytes=$1
    if [ $bytes -lt 1024 ]; then
        echo "${bytes}B"
    elif [ $bytes -lt 1048576 ]; then
        echo "$((bytes / 1024))KB"
    else
        echo "$((bytes / 1048576))MB"
    fi
}

# Função: verifica se um arquivo deve ser ignorado
should_ignore_file() {
    local filepath="$1"
    local filename=$(basename "$filepath")
    
    # Sempre ignorar .DS_Store
    if [[ "$filename" == ".DS_Store" ]]; then
        return 0
    fi
    
    # Converte pattern em array
    IFS='|' read -ra IGNORE_FILES_ARRAY <<< "$IGNORE_FILES_PATTERN"
    
    for pattern in "${IGNORE_FILES_ARRAY[@]}"; do
        # Remove espaços em branco
        pattern=$(echo "$pattern" | xargs)
        
        # Verifica se o pattern corresponde ao nome do arquivo
        if [[ "$filename" == $pattern ]]; then
            return 0  # Deve ignorar
        fi
    done
    
    return 1  # Não deve ignorar
}

# Função: verifica se um caminho deve ser ignorado
should_ignore_path() {
    local filepath="$1"
    local project_dir="$2"
    
    # Obtém o caminho absoluto do arquivo
    local absolute_filepath="$(realpath "$filepath" 2>/dev/null || echo "$filepath")"
    
    # Remove o diretório do projeto do caminho para comparação relativa
    local relative_path="${filepath#$project_dir/}"
    
    # Verifica caminhos ABSOLUTOS específicos
    if [ -n "$IGNORE_ABSOLUTE_PATHS" ]; then
        IFS='|' read -ra IGNORE_ABS_ARRAY <<< "$IGNORE_ABSOLUTE_PATHS"
        for ignore_path in "${IGNORE_ABS_ARRAY[@]}"; do
            ignore_path=$(echo "$ignore_path" | xargs)
            # Verifica se o caminho absoluto começa com o caminho a ignorar
            if [[ "$absolute_filepath" == "$ignore_path"* ]]; then
                return 0  # Deve ignorar
            fi
        done
    fi
    
    # Verifica caminhos RELATIVOS específicos
    if [ -n "$IGNORE_PATHS" ]; then
        IFS='|' read -ra IGNORE_PATHS_ARRAY <<< "$IGNORE_PATHS"
        for ignore_path in "${IGNORE_PATHS_ARRAY[@]}"; do
            ignore_path=$(echo "$ignore_path" | xargs)
            # Verifica se o caminho relativo contém o padrão
            if [[ "$relative_path" == *"$ignore_path"* ]]; then
                return 0  # Deve ignorar
            fi
        done
    fi
    
    return 1  # Não deve ignorar
}

########################################
# FUNÇÃO PRINCIPAL DE PROCESSAMENTO
########################################

process_project() {
    local project_dir="$1"
    local project_name="$2"
    local output_file="$3"
    
    echo "  📁 Processando: $project_name"
    
    # Limpa/cria arquivo de saída
    : > "$output_file"
    
    # Estatísticas
    local file_count=0
    local skipped_count=0
    local total_size=0
    
    ########################################
    # CABEÇALHO E ESTRUTURA DE DIRETÓRIOS
    ########################################
    {
        echo "╔=╗"
        echo "║ PROJETO: $project_name"
        echo "║ Data de geração: $(date '+%Y-%m-%d %H:%M:%S')"
        echo "╚=╝"
        echo
        echo "📂 ESTRUTURA DE DIRETÓRIOS"
        echo "="
        
        if command -v tree >/dev/null 2>&1; then
            # Usando tree, ignorando diretórios pesados e arquivos .DS_Store
            tree -a -I "$IGNORE_DIRS_PATTERN|.DS_Store|._*" "$project_dir" 2>/dev/null || echo "Erro ao gerar árvore de diretórios"
        else
            # Fallback com find
            IGN_LIST=(${IGNORE_DIRS_PATTERN//|/ })
            # head truncates to 500 lines and can raise SIGPIPE under pipefail, so ignore that non-zero exit.
            find "$project_dir" \
                \( -type d \( $(printf -- '-name %q -o ' "${IGN_LIST[@]}") -false \) -prune \) -o \
                \( -type f -name '.DS_Store' -prune \) -o \
                \( -type f -name '._*' -prune \) -o \
                -print 2>/dev/null | grep -v '\.DS_Store' | grep -v '\._' | sed "s|$project_dir|.|" | head -500 || true
        fi
        echo
        echo
    } >> "$output_file"
    
    ########################################
    # MONTA EXPRESSÕES PARA O FIND
    ########################################
    
    # Expressão para arquivos de código
    code_name_expr=()
    for ext in "${CODE_EXTS[@]}"; do
        code_name_expr+=( -name "*.${ext}" -o )
    done
    
    # Expressão para arquivos de configuração
    for cfg in "${CONFIG_FILES[@]}"; do
        code_name_expr+=( -name "$cfg" -o )
    done
    
    # Remove último "-o"
    if [ ${#code_name_expr[@]} -gt 0 ]; then
        unset 'code_name_expr[${#code_name_expr[@]}-1]'
    fi
    
    ########################################
    # COLETA E PROCESSA ARQUIVOS
    ########################################
    
    {
        echo "📄 CONTEÚDO DOS ARQUIVOS"
        echo "="
        echo
    } >> "$output_file"
    
    IGN_LIST=(${IGNORE_DIRS_PATTERN//|/ })
    
    # Debug: conta total de arquivos que serão processados
    local total_files=$(find "$project_dir" \
        \( -type d \( $(printf -- '-name %q -o ' "${IGN_LIST[@]}") -false \) -prune \) -o \
        -type f \( "${code_name_expr[@]}" \) -print 2>/dev/null | wc -l)
    
    echo "    📊 Total de arquivos encontrados: $total_files"
    
    find "$project_dir" \
        \( -type d \( $(printf -- '-name %q -o ' "${IGN_LIST[@]}") -false \) -prune \) -o \
        -type f \( "${code_name_expr[@]}" \) -print0 2>/dev/null \
        | sort -z \
        | while IFS= read -r -d '' filepath; do
            
            # Caminho relativo
            RELATIVE_PATH="./${filepath#$project_dir/}"
            
            # Verifica se o arquivo deve ser ignorado
            if should_ignore_file "$filepath"; then
                ((++skipped_count))
                continue
            fi
            
            # Verifica se o caminho deve ser ignorado
            if should_ignore_path "$filepath" "$project_dir"; then
                ((++skipped_count))
                continue
            fi
            
            # Obtém tamanho do arquivo
            SIZE_BYTES="$(get_size_bytes "$filepath")"
            SIZE_FORMATTED="$(format_bytes $SIZE_BYTES)"
            
            # Verifica limite de tamanho
            if [ "$SIZE_BYTES" -gt "$MAX_SIZE_BYTES" ]; then
                {
                    echo "┌─────────────────────────────────────────────────────────────"
                    echo "│ 📄 $RELATIVE_PATH"
                    echo "│ ⚠️  IGNORADO: Arquivo muito grande ($SIZE_FORMATTED > $(format_bytes $MAX_SIZE_BYTES))"
                    echo "└─────────────────────────────────────────────────────────────"
                    echo
                } >> "$output_file"
                ((++skipped_count))
                continue
            fi
            
            # Adiciona conteúdo do arquivo
            {
                echo "┌─────────────────────────────────────────────────────────────"
                echo "│ 📄 $RELATIVE_PATH"
                echo "│ 📊 Tamanho: $SIZE_FORMATTED"
                echo "├─────────────────────────────────────────────────────────────"
                
                # Verifica se é arquivo texto
                if file "$filepath" 2>/dev/null | grep -q "text\|ASCII\|UTF"; then
                    # Remove CR e adiciona números de linha
                    tr -d '\r' < "$filepath" 2>/dev/null | nl -ba -w4 -s' │ ' || echo "│ [Erro ao ler arquivo]"
                else
                    echo "│ [Arquivo binário - conteúdo omitido]"
                fi
                
                echo "└─────────────────────────────────────────────────────────────"
                echo
            } >> "$output_file"
            
            ((++file_count))
            total_size=$((total_size + SIZE_BYTES))
        done
    
    # Adiciona sumário ao final do arquivo
    {
        echo
        # echo "="
        # echo "📊 SUMÁRIO"
        # echo "="
        # echo "  ✅ Arquivos processados: $file_count"
        # echo "  ⏭️  Arquivos ignorados: $skipped_count"
        # echo "  💾 Tamanho total processado: $(format_bytes $total_size)"
        # echo "="
    } >> "$output_file"
    
    echo "    ✅ Arquivos processados: $file_count"
    echo "    ⏭️  Arquivos ignorados: $skipped_count"
    echo "    💾 Tamanho total: $(format_bytes $total_size)"
}

########################################
# SCRIPT PRINCIPAL
########################################

if [ -t 1 ] && [ -n "${TERM:-}" ] && command -v clear >/dev/null 2>&1; then
    clear
fi
echo "╔=╗"
echo "║           SCANNER DE PROJETOS DE CÓDIGO                   ║"
echo "╚=╝"
echo

# Verifica existência do diretório de destino
if [ ! -d "$TARGET_DIR" ]; then
    if [ "$TARGET_DIR" = "$DEFAULT_INPUT_DIR" ]; then
        mkdir -p "$TARGET_DIR"
        echo "ℹ️  O diretório padrão de entrada foi criado em: $TARGET_DIR"
        echo "   Adicione os projetos que deseja analisar dentro desse diretório e execute o script novamente."
        exit 0
    fi

    echo "❌ Erro: O diretório de destino não foi encontrado em: $TARGET_DIR" >&2
    exit 1
fi

# Cria diretório de saída
mkdir -p "$OUTPUT_DIR"

echo "📍 Configurações:"
echo "   • Diretório alvo: $TARGET_DIR"
echo "   • Diretório de saída: $OUTPUT_DIR"
echo "   • Tamanho máximo por arquivo: $(format_bytes $MAX_SIZE_BYTES)"
echo "   • Arquivos ignorados: $(echo "$IGNORE_FILES_PATTERN" | tr '|' ', ')"
echo "   • Diretórios ignorados: $(echo "$IGNORE_DIRS_PATTERN" | tr '|' ', ')"
[ -n "$IGNORE_PATHS" ] && echo "   • Caminhos relativos ignorados: $(echo "$IGNORE_PATHS" | tr '|' ', ')"
[ -n "$IGNORE_ABSOLUTE_PATHS" ] && echo "   • Caminhos absolutos ignorados: $(echo "$IGNORE_ABSOLUTE_PATHS" | tr '|' ', ')"
echo
echo "="
echo "🚀 Iniciando varredura..."
echo "="
echo

# Processa cada subdiretório do TARGET_DIR como um projeto separado
project_count=0

for project_path in "$TARGET_DIR"/*; do
    if [ -d "$project_path" ]; then
        project_name=$(basename "$project_path")
        output_file="$OUTPUT_DIR/${project_name}${OUTPUT_FILE_SUFFIX}"
        
        echo "[Projeto $((++project_count))]"
        process_project "$project_path" "$project_name" "$output_file"
        echo "  💾 Salvo em: $output_file"
        echo
    fi
done

# Se não houver subdiretórios, processa o TARGET_DIR inteiro
if [ $project_count -eq 0 ]; then
    echo "ℹ️  Nenhum subdiretório encontrado. Processando $TARGET_DIR como projeto único..."
    echo
    
    project_name=$(basename "$TARGET_DIR")
    output_file="$OUTPUT_DIR/${project_name}${OUTPUT_FILE_SUFFIX}"
    
    process_project "$TARGET_DIR" "$project_name" "$output_file"
    echo "  💾 Salvo em: $output_file"
fi

echo
echo "="
echo "✨ CONCLUÍDO!"
echo "="
echo "  📊 Total de projetos processados: $project_count"
echo "  📂 Arquivos gerados em: $OUTPUT_DIR"
echo "="
echo

# Lista os arquivos gerados
echo "📋 Arquivos gerados:"
echo "────────────────────────────────────────────────────────────"
ls -lh "$OUTPUT_DIR"/*${OUTPUT_FILE_SUFFIX} 2>/dev/null || echo "  ⚠️  Nenhum arquivo gerado."
echo

# Mensagem de uso das variáveis de ambiente
echo "💡 Dica: Você pode personalizar o comportamento usando variáveis de ambiente:"
echo "   • TARGET_DIR - Diretório a escanear"
echo "   • OUTPUT_DIR - Diretório de saída"
echo "   • OUTPUT_FILE_SUFFIX - Sufixo dos arquivos"
echo "   • MAX_SIZE_BYTES - Tamanho máximo por arquivo"
echo "   • IGNORE_FILES_EXTRA - Arquivos adicionais a ignorar"
echo "   • IGNORE_DIRS_EXTRA - Diretórios adicionais a ignorar"
echo "   • IGNORE_PATHS - Caminhos relativos específicos a ignorar"
echo "   • IGNORE_ABSOLUTE_PATHS - Caminhos absolutos específicos a ignorar"
echo
echo "📌 EXEMPLOS DE USO:"
echo "────────────────────────────────────────────────────────────"
echo
echo "1️⃣  Ignorar caminhos ABSOLUTOS:"
echo '   IGNORE_ABSOLUTE_PATHS="$PWD/input/vendor/symfony|$PWD/input/libs/huge" ./$(basename $0)'
echo
echo "2️⃣  Ignorar caminhos RELATIVOS (dentro do projeto):"
echo '   IGNORE_PATHS="src/vendor/large-lib|tests/fixtures/big-data" ./$(basename $0)'
echo
echo "3️⃣  Usar ambos simultaneamente:"
echo '   IGNORE_ABSOLUTE_PATHS="$PWD/input/legacy" \'
echo '   IGNORE_PATHS="vendor/old-packages|cache/temp" \'
echo '   ./$(basename $0)'
echo
echo "4️⃣  Exemplo completo:"
echo '   export TARGET_DIR="./custom-input"'
echo '   export IGNORE_ABSOLUTE_PATHS="$PWD/custom-input/huge-library"'
echo '   export IGNORE_PATHS="frontend/build|backend/vendor/aws-sdk"'
echo '   export IGNORE_DIRS_EXTRA="backup|legacy"'
echo '   export IGNORE_FILES_EXTRA="*.backup|*.tmp"'
echo '   ./$(basename $0)'
echo
