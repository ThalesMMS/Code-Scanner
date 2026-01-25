#!/usr/bin/env bash
set -euo pipefail

########################################
# UNIFIED BASH CODE SCANNER
# Combines legacy, enhanced, and Windows-friendly flows into one script.
########################################

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

DEFAULT_INPUT_DIR="$REPO_ROOT/input"
DEFAULT_OUTPUT_DIR="$REPO_ROOT/output"

########################################
# CONFIGURAÇÃO PRINCIPAL
########################################

TARGET_DIR="${TARGET_DIR:-$DEFAULT_INPUT_DIR}"
OUTPUT_DIR="${OUTPUT_DIR:-$DEFAULT_OUTPUT_DIR}"
OUTPUT_FILE_SUFFIX="${OUTPUT_FILE_SUFFIX:-_project_code.txt}"
MAX_SIZE_BYTES="${MAX_SIZE_BYTES:-2097152}"
USE_GITIGNORE="${USE_GITIGNORE:-true}"
VERBOSE="${VERBOSE:-false}"

########################################
# LISTAS DE EXCLUSÃO
########################################

IGNORE_FILES_BASE='.DS_Store|Thumbs.db|.env|.env.local|.env.production|.env.development|*.key|*.pem|*.p12|*.pfx|*.log|*.pid|*.seed|*.sqlite|*.sqlite3|*.db|desktop.ini|*.swp|*.swo|*~|.~lock.*|._*'
IGNORE_FILES_EXTRA="${IGNORE_FILES_EXTRA:-}"

IGNORE_DIRS_BASE='node_modules|dist|build|target|bin|.git|.next|coverage|.turbo|.cache|.expo|.gradle|.mvn|.settings|Pods|DerivedData|.idea|.vscode|out|tmp|.parcel-cache|.sass-cache|.nuxt|.svelte-kit|__pycache__|.pytest_cache|cmake-build-debug|cmake-build-release|CMakeFiles|.dart_tool|.pub-cache|.pub|.android|.ios|.macos|.windows|.linux|.metadata|.packages|.flutter-plugins|.flutter-plugins-dependencies|vendor|bower_components|jspm_packages|web_modules|.yarn|.pnp.*|venv|env|virtualenv|.tox|.mypy_cache|htmlcov'
IGNORE_DIRS_EXTRA="${IGNORE_DIRS_EXTRA:-}"

IGNORE_PATHS="${IGNORE_PATHS:-}"
IGNORE_ABSOLUTE_PATHS="${IGNORE_ABSOLUTE_PATHS:-}"

IGNORE_FILES_PATTERN="${IGNORE_FILES_BASE}${IGNORE_FILES_EXTRA:+|$IGNORE_FILES_EXTRA}"
IGNORE_DIRS_PATTERN="${IGNORE_DIRS_BASE}${IGNORE_DIRS_EXTRA:+|$IGNORE_DIRS_EXTRA}"

########################################
# EXTENSÕES E ARQUIVOS
########################################

CODE_EXTS=(
    # Web / Front-end
    "js" "jsx" "mjs" "cjs" "ts" "tsx" "mts" "cts" "vue"
    "html" "htm" "css" "scss" "sass" "less"

    # Backend / Scripting
    "py" "pyx" "pyi"
    "java" "kt" "kts"
    "rs"
    "go"
    "rb" "erb"
    "php"
    "cs" "fs" "vb"

    # Systems / Mobile
    "c" "cpp" "cc" "cxx" "c++" "h" "hpp" "hxx" "h++" "hh"
    "m" "mm" "swift"
    "dart"

    # Markup / Config-like
    "md" "mdx" "markdown"
    "json" "yaml" "yml" "toml" "xml"
    "sh" "bash" "zsh" "fish"

    # Outros
    "metal" "sql"
)

CONFIG_FILES=(
    # Node/TS/Bundlers
    "package.json" "package-lock.json" "pnpm-lock.yaml" "yarn.lock" "bun.lockb"
    "tsconfig.json" "tsconfig.*.json" "jsconfig.json"
    "vite.config.*" "webpack.config.*" "rollup.config.*"
    "babel.config.*" ".babelrc*"
    "next.config.*" ".eslintrc*" ".prettierrc*" "prettier.config.*"
    "postcss.config.*" "tailwind.config.*" "stylelint.config.*" ".stylelintrc*"
    ".npmrc" ".nvmrc" ".node-version"

    # Java/Maven/Gradle
    "pom.xml" "build.gradle*" "settings.gradle*" "gradle.properties"
    "gradlew" "gradlew.bat" "mvnw" "mvnw.cmd"
    ".classpath" ".project"

    # Spring Boot
    "application*.properties" "application*.yml" "application*.yaml"
    "bootstrap*.properties" "bootstrap*.yml" "bootstrap*.yaml"

    # Python
    "requirements.txt" "setup.py" "setup.cfg" "pyproject.toml"
    "Pipfile" "Pipfile.lock" "poetry.lock" "tox.ini" "pytest.ini"

    # Ruby
    "Gemfile" "Gemfile.lock" "Rakefile"

    # Rust
    "Cargo.toml" "Cargo.lock"

    # Go
    "go.mod" "go.sum"

    # .NET
    "*.csproj" "*.sln" "*.fsproj"

    # PHP
    "composer.json" "composer.lock"

    # Build / Docker / Tools
    "Makefile" "makefile" "GNUmakefile" "CMakeLists.txt" "*.cmake"
    "configure" "configure.ac" "configure.in"
    "*.pro" "*.pri"
    "meson.build" "meson_options.txt"
    "BUILD" "BUILD.bazel" "WORKSPACE"
    "Dockerfile" "docker-compose.yml" "docker-compose.yaml"
    ".dockerignore" "Procfile"

    # Flutter / Mobile
    "pubspec.yaml" "pubspec.lock" "pubspec_overrides.yaml"
    ".metadata" ".packages" ".dart_tool/package_config.json"
    "analysis_options.yaml" "*.iml"
    "android/local.properties" "ios/Podfile*"

    # Apple / Xcode
    "*.xcodeproj" "*.xcworkspace" "*.xcscheme" "*.pbxproj"
    "Info.plist" "Entitlements.plist"

    # Documentação
    "README*" "LICENSE*" ".gitignore" ".gitattributes"
)

########################################
# FUNÇÕES AUXILIARES
########################################

log_verbose() {
    if [ "$VERBOSE" = "true" ]; then
        echo "  [VERBOSE] $*" >&2
    fi
}

get_size_bytes() {
    local f="$1"
    if size=$(stat -f%z "$f" 2>/dev/null); then
        echo "$size"
    else
        stat -c%s "$f" 2>/dev/null || echo "0"
    fi
}

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

should_ignore_file() {
    local filepath="$1"
    local filename=$(basename "$filepath")

    # Sempre ignore .DS_Store
    if [[ "$filename" == ".DS_Store" ]]; then
        log_verbose "Ignoring $filename (system file)"
        return 0
    fi

    IFS='|' read -ra IGNORE_FILES_ARRAY <<< "$IGNORE_FILES_PATTERN"
    for pattern in "${IGNORE_FILES_ARRAY[@]}"; do
        pattern=$(echo "$pattern" | xargs)
        if [[ "$filename" == $pattern ]]; then
            log_verbose "Ignoring $filename (pattern: $pattern)"
            return 0
        fi
    done

    return 1
}

should_ignore_path() {
    local filepath="$1"
    local project_dir="$2"

    local absolute_filepath="$(realpath "$filepath" 2>/dev/null || echo "$filepath")"
    local relative_path="${filepath#$project_dir/}"

    if [ -n "$IGNORE_ABSOLUTE_PATHS" ]; then
        IFS='|' read -ra IGNORE_ABS_ARRAY <<< "$IGNORE_ABSOLUTE_PATHS"
        for ignore_path in "${IGNORE_ABS_ARRAY[@]}"; do
            ignore_path=$(echo "$ignore_path" | xargs)
            if [[ "$absolute_filepath" == "$ignore_path"* ]]; then
                log_verbose "Ignoring $relative_path (absolute path match)"
                return 0
            fi
        done
    fi

    if [ -n "$IGNORE_PATHS" ]; then
        IFS='|' read -ra IGNORE_PATHS_ARRAY <<< "$IGNORE_PATHS"
        for ignore_path in "${IGNORE_PATHS_ARRAY[@]}"; do
            ignore_path=$(echo "$ignore_path" | xargs)
            if [[ "$relative_path" == *"$ignore_path"* ]]; then
                log_verbose "Ignoring $relative_path (relative path match)"
                return 0
            fi
        done
    fi

    return 1
}

check_gitignore() {
    local filepath="$1"
    local project_dir="$2"
    local gitignore="$project_dir/.gitignore"

    if [ "$USE_GITIGNORE" != "true" ] || [ ! -f "$gitignore" ]; then
        return 1
    fi

    local relative_path="${filepath#$project_dir/}"

    while IFS= read -r pattern; do
        [[ -z "$pattern" ]] && continue
        [[ "$pattern" =~ ^#.* ]] && continue

        pattern=$(echo "$pattern" | xargs)

        if [[ "$relative_path" == *"$pattern"* ]] || [[ $(basename "$filepath") == $pattern ]]; then
            log_verbose "Ignoring $relative_path (.gitignore: $pattern)"
            return 0
        fi
    done < "$gitignore"

    return 1
}

detect_project_type() {
    local project_dir="$1"
    local types=()

    if [ -f "$project_dir/package.json" ]; then
        types+=("Node.js")
    fi

    if [ -f "$project_dir/requirements.txt" ] || [ -f "$project_dir/setup.py" ] || [ -f "$project_dir/pyproject.toml" ]; then
        types+=("Python")
    fi

    if [ -f "$project_dir/manage.py" ]; then
        types+=("Django")
    fi

    if [ -f "$project_dir/pom.xml" ]; then
        types+=("Maven")
    fi

    if [ -f "$project_dir/build.gradle" ]; then
        types+=("Gradle")
    fi

    if [ -f "$project_dir/Cargo.toml" ]; then
        types+=("Rust")
    fi

    if [ -f "$project_dir/go.mod" ]; then
        types+=("Go")
    fi

    if [ -n "$(find "$project_dir" -maxdepth 1 -name "*.csproj" -o -name "*.sln" 2>/dev/null)" ]; then
        types+=(".NET")
    fi

    if [ -f "$project_dir/pubspec.yaml" ] && [ -d "$project_dir/lib" ]; then
        types+=("Flutter")
    fi

    if [ -f "$project_dir/Dockerfile" ]; then
        types+=("Docker")
    fi

    if [ ${#types[@]} -eq 0 ]; then
        echo "Generic"
    else
        IFS=", "
        echo "${types[*]}"
    fi
}

########################################
# FUNÇÃO PRINCIPAL DE PROCESSAMENTO
########################################

process_project() {
    local project_dir="$1"
    local project_name="$2"
    local output_file="$3"

    echo "  📁 Processando: $project_name"

    local project_type=$(detect_project_type "$project_dir")
    echo "    🔍 Tipo detectado: $project_type"

    if [ -f "$project_dir/.gitignore" ] && [ "$USE_GITIGNORE" = "true" ]; then
        echo "    📋 Usando .gitignore do projeto"
    fi

    : > "$output_file"

    local file_count=0
    local skipped_count=0
    local total_size=0
    local gitignore_count=0

    {
        echo "$project_name"
        echo "$(date '+%Y-%m-%d %H:%M:%S')"
        echo
        echo "FOLDER STRUCTURE"

        if command -v tree >/dev/null 2>&1; then
            tree -a -I "$IGNORE_DIRS_PATTERN|.DS_Store|._*" "$project_dir" 2>/dev/null || echo "Erro ao gerar árvore"
        else
            IFS='|' read -ra IGN_LIST <<< "$IGNORE_DIRS_PATTERN"
            # head pode gerar SIGPIPE sob pipefail; ignore status não zero
            find "$project_dir" \
                \( -type d \( $(printf -- '-name %q -o ' "${IGN_LIST[@]}") -false \) -prune \) -o \
                \( -type f -name '.DS_Store' -prune \) -o \
                \( -type f -name '._*' -prune \) -o \
                -print 2>/dev/null | grep -v '\\.DS_Store' | grep -v '\\._' | sed "s|$project_dir|.|" | head -500 || true
        fi
        echo
        echo
    } >> "$output_file"

    code_name_expr=()
    for ext in "${CODE_EXTS[@]}"; do
        code_name_expr+=( -name "*.${ext}" -o )
    done
    for cfg in "${CONFIG_FILES[@]}"; do
        code_name_expr+=( -name "$cfg" -o )
    done
    if [ ${#code_name_expr[@]} -gt 0 ]; then
        unset 'code_name_expr[${#code_name_expr[@]}-1]'
    fi

    {
        echo "📄 CONTEÚDO DOS ARQUIVOS"
        echo "═══════════════════════════════════════════════════════════════"
        echo
    } >> "$output_file"

    IFS='|' read -ra IGN_LIST <<< "$IGNORE_DIRS_PATTERN"

    local total_files=$(find "$project_dir" \
        \( -type d \( $(printf -- '-name %q -o ' "${IGN_LIST[@]}") -false \) -prune \) -o \
        -type f \( "${code_name_expr[@]}" \) -print 2>/dev/null | wc -l)

    echo "    📊 Arquivos encontrados: $total_files"

    find "$project_dir" \
        \( -type d \( $(printf -- '-name %q -o ' "${IGN_LIST[@]}") -false \) -prune \) -o \
        -type f \( "${code_name_expr[@]}" \) -print0 2>/dev/null \
        | sort -z \
        | while IFS= read -r -d '' filepath; do

            RELATIVE_PATH="./${filepath#$project_dir/}"

            if check_gitignore "$filepath" "$project_dir"; then
                ((gitignore_count++))
                ((skipped_count++))
                continue
            fi

            if should_ignore_file "$filepath"; then
                ((skipped_count++))
                continue
            fi

            if should_ignore_path "$filepath" "$project_dir"; then
                ((skipped_count++))
                continue
            fi

            SIZE_BYTES="$(get_size_bytes "$filepath")"
            SIZE_FORMATTED="$(format_bytes $SIZE_BYTES)"

            if [ "$SIZE_BYTES" -gt "$MAX_SIZE_BYTES" ]; then
                if [ "$VERBOSE" = "true" ]; then
                    {
                        echo "┌─────────────────────────────────────────────────────────────"
                        echo "│ 📄 $RELATIVE_PATH"
                        echo "│ ⚠️  IGNORADO: Muito grande ($SIZE_FORMATTED > $(format_bytes $MAX_SIZE_BYTES))"
                        echo "└─────────────────────────────────────────────────────────────"
                        echo
                    } >> "$output_file"
                else
                    {
                        echo "📄 $RELATIVE_PATH"
                        echo "⚠️  IGNORADO: Muito grande ($SIZE_FORMATTED > $(format_bytes $MAX_SIZE_BYTES))"
                        echo
                    } >> "$output_file"
                fi
                ((skipped_count++))
                continue
            fi

            if [ "$VERBOSE" = "true" ]; then
                {
                    echo "┌─────────────────────────────────────────────────────────────"
                    echo "│ 📄 $RELATIVE_PATH"
                    echo "│ 📊 Tamanho: $SIZE_FORMATTED"
                    echo "├─────────────────────────────────────────────────────────────"

                    if file "$filepath" 2>/dev/null | grep -q "text\\|ASCII\\|UTF"; then
                        tr -d '\r' < "$filepath" 2>/dev/null | nl -ba -w4 -s' │ ' || echo "│ [Erro ao ler]"
                    else
                        echo "│ [Arquivo binário - omitido]"
                    fi

                    echo "└─────────────────────────────────────────────────────────────"
                    echo
                } >> "$output_file"
            else
                {
                    echo "📄 $RELATIVE_PATH"
                    if file "$filepath" 2>/dev/null | grep -q "text\\|ASCII\\|UTF"; then
                        tr -d '\r' < "$filepath" 2>/dev/null || echo "[Erro ao ler]"
                    else
                        echo "[Arquivo binário - omitido]"
                    fi
                    echo
                } >> "$output_file"
            fi

            ((file_count++))
            total_size=$((total_size + SIZE_BYTES))
        done

    if [ "$VERBOSE" = "true" ]; then
        {
            echo
            echo "═══════════════════════════════════════════════════════════════"
            echo "📊 RESUMO"
            echo "═══════════════════════════════════════════════════════════════"
            echo "  ✅ Arquivos processados: $file_count"
            echo "  ⏭️  Arquivos ignorados: $skipped_count"
            [ $gitignore_count -gt 0 ] && echo "  📋 Ignorados via .gitignore: $gitignore_count"
            echo "  💾 Tamanho total: $(format_bytes $total_size)"
            echo "═══════════════════════════════════════════════════════════════"
        } >> "$output_file"
    fi

    echo "    ✅ Processados: $file_count"
    echo "    ⏭️  Ignorados: $skipped_count"
    [ $gitignore_count -gt 0 ] && echo "    📋 Via .gitignore: $gitignore_count"
    echo "    💾 Tamanho: $(format_bytes $total_size)"
}

########################################
# SCRIPT PRINCIPAL
########################################

if [ -t 1 ] && [ -n "${TERM:-}" ] && command -v clear >/dev/null 2>&1; then
    clear
fi

echo "╔═══════════════════════════════════════════════════════════════╗"
echo "║                SCANNER DE PROJETOS DE CÓDIGO                  ║"
echo "╚═══════════════════════════════════════════════════════════════╝"
echo

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

mkdir -p "$OUTPUT_DIR"

echo "📍 Configurações:"
echo "   • Diretório alvo: $TARGET_DIR"
echo "   • Diretório de saída: $OUTPUT_DIR"
echo "   • Sufixo de saída: $OUTPUT_FILE_SUFFIX"
echo "   • Tamanho máximo por arquivo: $(format_bytes $MAX_SIZE_BYTES)"
echo "   • Usar .gitignore: $USE_GITIGNORE"
echo "   • Modo verbose: $VERBOSE"
[ -n "$IGNORE_FILES_EXTRA" ] && echo "   • Arquivos extra ignorados: $(echo "$IGNORE_FILES_EXTRA" | tr '|' ', ')"
[ -n "$IGNORE_DIRS_EXTRA" ] && echo "   • Diretórios extra ignorados: $(echo "$IGNORE_DIRS_EXTRA" | tr '|' ', ')"
[ -n "$IGNORE_PATHS" ] && echo "   • Caminhos relativos ignorados: $(echo "$IGNORE_PATHS" | tr '|' ', ')"
[ -n "$IGNORE_ABSOLUTE_PATHS" ] && echo "   • Caminhos absolutos ignorados: $(echo "$IGNORE_ABSOLUTE_PATHS" | tr '|' ', ')"

echo
echo "═══════════════════════════════════════════════════════════════"
echo "🚀 Iniciando varredura..."
echo "═══════════════════════════════════════════════════════════════"

project_count=0

for project_path in "$TARGET_DIR"/*; do
    if [ -d "$project_path" ]; then
        project_name=$(basename "$project_path")
        output_file="$OUTPUT_DIR/${project_name}${OUTPUT_FILE_SUFFIX}"

        echo "[Projeto $((++project_count))]"
        process_project "$project_path" "$project_name" "$output_file"
        echo "  💾 Salvo: $output_file"
        echo
    fi
done

if [ $project_count -eq 0 ]; then
    echo "ℹ️  Nenhum subdiretório encontrado. Processando $TARGET_DIR como projeto único..."
    echo

    project_name=$(basename "$TARGET_DIR")
    output_file="$OUTPUT_DIR/${project_name}${OUTPUT_FILE_SUFFIX}"

    process_project "$TARGET_DIR" "$project_name" "$output_file"
    echo "  💾 Salvo: $output_file"
fi

echo
echo "═══════════════════════════════════════════════════════════════"
echo "✨ CONCLUÍDO!"
echo "═══════════════════════════════════════════════════════════════"
echo "  📊 Total de projetos processados: $project_count"
echo "  📂 Arquivos gerados em: $OUTPUT_DIR"
echo "═══════════════════════════════════════════════════════════════"
echo

echo "📋 Arquivos gerados:"
ls -lh "$OUTPUT_DIR"/*${OUTPUT_FILE_SUFFIX} 2>/dev/null || echo "  ⚠️  Nenhum arquivo gerado."
echo

echo "💡 Variáveis de ambiente disponíveis:"
echo "   • TARGET_DIR - Diretório a escanear"
echo "   • OUTPUT_DIR - Diretório de saída"
echo "   • OUTPUT_FILE_SUFFIX - Sufixo dos arquivos"
echo "   • MAX_SIZE_BYTES - Tamanho máximo por arquivo"
echo "   • USE_GITIGNORE - Usar .gitignore (true/false)"
echo "   • VERBOSE - Modo detalhado (true/false)"
echo "   • IGNORE_FILES_EXTRA - Arquivos adicionais a ignorar (|' separados)"
echo "   • IGNORE_DIRS_EXTRA - Diretórios adicionais a ignorar (|' separados)"
echo "   • IGNORE_PATHS - Caminhos relativos específicos a ignorar (|' separados)"
echo "   • IGNORE_ABSOLUTE_PATHS - Caminhos absolutos específicos a ignorar (|' separados)"
echo

echo "📌 Exemplos rápidos:"
echo "   IGNORE_ABSOLUTE_PATHS=\"$PWD/input/vendor/symfony|$PWD/input/libs/huge\" ./bash/scan_project.sh"
echo "   IGNORE_PATHS=\"src/vendor/large-lib|tests/fixtures/big-data\" ./bash/scan_project.sh"
echo "   USE_GITIGNORE=false VERBOSE=true TARGET_DIR=./custom OUTPUT_DIR=./reports ./bash/scan_project.sh"
echo
