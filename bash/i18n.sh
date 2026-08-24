# i18n helpers for bash/scan_project.sh — reads i18n/<locale>.ftl (same keys as en-US).

i18n_resolve_locale() {
    local requested="${CODE_SCANNER_LANG:-}"
    local os_lang="${LANG:-}"
    local catalog_dir="$REPO_ROOT/i18n"
    local fallback="en-US"

    _i18n_match() {
        local tag="${1//_/-}"
        tag="${tag%%.*}"
        [ -z "$tag" ] && return 1
        [ -f "$catalog_dir/${tag}.ftl" ] && { printf '%s' "$tag"; return 0; }
        local lang="${tag%%-*}"
        local match
        match="$(find "$catalog_dir" -maxdepth 1 -name "${lang}-*.ftl" -print 2>/dev/null | head -1)"
        if [ -n "$match" ]; then
            basename "$match" .ftl
            return 0
        fi
        return 1
    }

    local resolved
    if resolved="$(_i18n_match "$requested")"; then
        printf '%s' "$resolved"
        return
    fi
    if resolved="$(_i18n_match "$os_lang")"; then
        printf '%s' "$resolved"
        return
    fi
    printf '%s' "$fallback"
}

i18n_lookup() {
    local key="$1"
    local file="$2"
    [ -f "$file" ] || return 1
    awk -v key="$key" '
        $0 ~ "^" key "[[:space:]]*=" {
            sub(/^[^=]*=[[:space:]]*/, "")
            print
            found=1
            exit
        }
        END { exit found ? 0 : 1 }
    ' "$file"
}

i18n_t() {
    local key="$1"
    shift
    local text
    text="$(i18n_lookup "$key" "$I18N_FILE" || true)"
    if [ -z "$text" ]; then
        text="$(i18n_lookup "$key" "$I18N_FALLBACK_FILE" || true)"
    fi
    if [ -z "$text" ]; then
        printf '%s' "$key"
        return
    fi
    local pair name value
    for pair in "$@"; do
        name="${pair%%=*}"
        value="${pair#*=}"
        text="${text//\{ \$$name \}/$value}"
        text="${text//\{\$$name\}/$value}"
    done
    printf '%s' "$text"
}

I18N_LOCALE="$(i18n_resolve_locale)"
I18N_FILE="$REPO_ROOT/i18n/${I18N_LOCALE}.ftl"
I18N_FALLBACK_FILE="$REPO_ROOT/i18n/en-US.ftl"

# Numeric / date formatting mirrors src/i18n.rs (SI 1000, locale separators).
# Scoped LC_* on the date command only — never export LC_ALL for the whole script.

i18n_lang_subtag() {
    local tag="${I18N_LOCALE%%-*}"
    printf '%s' "$(printf '%s' "$tag" | tr '[:upper:]' '[:lower:]')"
}

i18n_decimal_sep() {
    case "$(i18n_lang_subtag)" in
        en|ja|ko|zh|th|he|id|ms|fil|hi|ta|te|mr|kn|gu|pa|ml|bn|sw|am|my|lo|km) printf '.' ;;
        *) printf ',' ;;
    esac
}

i18n_thousands_sep() {
    if [ "$(i18n_decimal_sep)" = ',' ]; then
        printf '.'
    else
        printf ','
    fi
}

i18n_format_size() {
    awk -v bytes="${1:-0}" -v dec="$(i18n_decimal_sep)" -v thou="$(i18n_thousands_sep)" '
        function group_int(value,    digits, out, len, i, neg) {
            neg = ""
            if (value < 0) { neg = "-"; value = -value }
            digits = sprintf("%.0f", value)
            out = ""
            len = length(digits)
            for (i = len; i > 0; i--) {
                out = substr(digits, i, 1) out
                if ((len - i + 1) % 3 == 0 && i > 1) out = thou out
            }
            return neg out
        }
        BEGIN {
            units[0] = "B"; units[1] = "kB"; units[2] = "MB"; units[3] = "GB"; units[4] = "TB"
            amount = bytes + 0
            if (amount < 1000) {
                printf "%s %s", group_int(amount), units[0]
                exit
            }
            unit = 0
            while (amount >= 1000 && unit < 4) {
                amount = amount / 1000
                unit++
            }
            text = sprintf("%.2f", amount)
            sub(/0+$/, "", text)
            sub(/\.$/, "", text)
            if (dec != ".") {
                sub(/\./, dec, text)
            }
            printf "%s %s", text, units[unit]
        }
    '
}

i18n_posix_time_locale() {
    local tag="${I18N_LOCALE//-/_}"
    local lang="${tag%%_*}"
    command -v locale >/dev/null 2>&1 || return 1
    locale -a 2>/dev/null | awk -v tag="$tag" -v lang="$lang" '
        {
            raw = $0
            base = raw
            sub(/\..*/, "", base)
            sub(/@.*/, "", base)
            gsub(/-/, "_", base)
            lraw = tolower(raw)
            lbase = tolower(base)
            ltag = tolower(tag)
            llang = tolower(lang)
        }
        lbase == ltag {
            exact = raw
            if (lraw ~ /\.utf-?8$/) { print raw; found = 1; exit }
            if (!utf) utf = raw
        }
        !lang_utf && lbase == llang && lraw ~ /\.utf-?8$/ { lang_utf = raw }
        !lang_any && lbase == llang { lang_any = raw }
        END {
            if (found) exit
            if (utf) { print utf; exit }
            if (exact) { print exact; exit }
            if (lang_utf) { print lang_utf; exit }
            if (lang_any) { print lang_any; exit }
            exit 1
        }
    '
}

i18n_format_timestamp() {
    local posix formatted
    posix="$(i18n_posix_time_locale || true)"
    if [ -n "$posix" ]; then
        formatted="$(LC_ALL="$posix" date '+%x %X' 2>/dev/null || true)"
        if [ -n "$formatted" ]; then
            printf '%s' "$formatted"
            return
        fi
    fi
    date '+%Y-%m-%d %H:%M:%S'
}
