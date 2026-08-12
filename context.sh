#!/bin/bash

# --- Configuration ---
ADDITIONAL_IGNORE=("/.git/" "/.sqlx/" "*.log" "/target/" "/node_modules/")

# --- Initialisation ---
PROJECT_ROOT=$(git rev-parse --show-toplevel 2>/dev/null || pwd)
cd "$PROJECT_ROOT" || exit 1

IS_GIT_REPO=false
if git rev-parse --is-inside-work-tree >/dev/null 2>&1; then
    IS_GIT_REPO=true
fi

TMP_FILE=$(mktemp)

# --- Helper: Syntax Highlighting Language ---
get_lang() {
    local file="$1"
    local filename
    filename=$(basename "$file")

    case "$filename" in
        .gitignore|gitignore) echo "gitignore" ;;
        Cargo.lock) echo "toml" ;;
        Makefile) echo "makefile" ;;
        Dockerfile) echo "dockerfile" ;;
        *)
            local ext="${filename##*.}"
            case "$ext" in
                rs) echo "rust" ;;
                sh|bash) echo "bash" ;;
                toml) echo "toml" ;;
                json) echo "json" ;;
                yaml|yml) echo "yaml" ;;
                md) echo "markdown" ;;
                js) echo "javascript" ;;
                ts) echo "typescript" ;;
                py) echo "python" ;;
                c|h) echo "c" ;;
                cpp|hpp) echo "cpp" ;;
                html) echo "html" ;;
                css) echo "css" ;;
                sql) echo "sql" ;;
                *) echo "" ;;
            esac
            ;;
    esac
}

# --- Helper: Check Binary File ---
is_binary() {
    # 0-byte files are empty text files, not binary
    [ ! -s "$1" ] && return 1

    if command -v file >/dev/null 2>&1; then
        file -b --mime-encoding "$1" | grep -qv "utf-8\|us-ascii\|iso-8859\|ascii"
    else
        grep -qI . "$1" 2>/dev/null && return 1 || return 0
    fi
}

# --- Helper: Check Ignore ---
is_ignored() {
    local raw_path="$1"
    local clean_path="${raw_path#./}"

    if [ "$clean_path" = "." ] || [ -z "$clean_path" ]; then
        return 1
    fi

    # 1. Check gitignore
    if [ "$IS_GIT_REPO" = true ]; then
        if git check-ignore -q "$raw_path" 2>/dev/null || git check-ignore -q "$clean_path" 2>/dev/null; then
            return 0
        fi
    fi

    # 2. Check additional ignore patterns
    local padded_path="/${clean_path%/}/"
    for pattern in "${ADDITIONAL_IGNORE[@]}"; do
        if [[ "$padded_path" == *"$pattern"* ]] || [[ "$clean_path" == $pattern ]]; then
            return 0
        fi
    done

    return 1
}

# --- Helper: List Files in Directory ---
get_files_in_dir() {
    local dir="$1"
    if [ "$IS_GIT_REPO" = true ]; then
        git ls-files --cached --others --exclude-standard "$dir" 2>/dev/null
    else
        find "$dir" -type f
    fi
}

# --- Helper: Append File Content Safely ---
append_file_content() {
    local file="$1"
    local rel_path="${file#$PROJECT_ROOT/}"
    local lang
    lang=$(get_lang "$file")

    {
        echo "## File: $rel_path"
        echo '```'"$lang"
        cat "$file"
        # Force a newline if the file is missing a trailing newline
        [ -n "$(tail -c1 "$file")" ] && echo ""
        echo '```'
        echo ""
    } >> "$TMP_FILE"
}

# --- 1. Process Clipboard (Wayland) ---
RAW_DATA=$(wl-paste -t text/uri-list 2>/dev/null)

if [ -z "$RAW_DATA" ]; then
    RAW_DATA=$(wl-paste 2>/dev/null)
fi

# Decode URL encoding (like %20) and remove file:// prefix
DECODED_DATA=$(echo "$RAW_DATA" | sed 's/file:\/\///g' | sed 's/\r//g' | perl -pe 's/%([0-9a-f]{2})/chr(hex($1))/eig')

FOUND_ANY=false

while IFS= read -r item; do
    item=$(echo "$item" | sed 's/^[[:space:]]*//;s/[[:space:]]*$//')

    if [ -e "$item" ]; then
        if [ "$FOUND_ANY" = false ]; then
            echo "📂 Project root: $PROJECT_ROOT"
            echo "Reading clipboard items..."
            FOUND_ANY=true
        fi

        if [ -d "$item" ]; then
            echo "📁 Processing directory: $item"

            while IFS= read -r subfile; do
                if [ -f "$subfile" ] && ! is_ignored "$subfile"; then
                    if is_binary "$subfile"; then
                        echo "  ⚠️ Skipped binary file: ${subfile#$PROJECT_ROOT/}"
                        continue
                    fi
                    append_file_content "$subfile"
                    echo "  + Added: ${subfile#$PROJECT_ROOT/}"
                fi
            done < <(get_files_in_dir "$item")

        elif [ -f "$item" ]; then
            if ! is_ignored "$item"; then
                if is_binary "$item"; then
                    echo "⚠️ Skipped binary file: ${item#$PROJECT_ROOT/}"
                    continue
                fi
                append_file_content "$item"
                echo "✔ Added: ${item#$PROJECT_ROOT/}"
            fi
        fi
    fi
done <<< "$DECODED_DATA"

if [ "$FOUND_ANY" = false ]; then
    echo "ℹ️  No files found on clipboard. (Clipboard contains plain text or is empty)"
fi

# --- 2. Folder Structure ---
read -p "Add folder structure? (y/N): " add_tree
if [[ "$add_tree" =~ ^[Yy]$ ]]; then
    STRICT_TMP=$(mktemp)
    {
        echo "## File & Directory Structure"
        echo '```'
        find . -maxdepth 3 -not -path '*/.*' | sort | while read -r line; do
            if ! is_ignored "$line"; then
                echo "$line"
            fi
        done | sed -e 's/[^-][^\/]*\// |/g' -e 's/|/|-- /g'
        echo '```'
        echo ""
    } > "$STRICT_TMP"
    cat "$TMP_FILE" >> "$STRICT_TMP"
    mv "$STRICT_TMP" "$TMP_FILE"
    echo "✔ Added structure."
fi

# --- 3. Git Diff ---
read -p "Add git diff? (y/N): " add_diff
if [[ "$add_diff" =~ ^[Yy]$ ]]; then
    DIFF=$(git diff)
    if [ -n "$DIFF" ]; then
        { echo "## Git Diff"; echo '```diff'; echo "$DIFF"; echo '```'; } >> "$TMP_FILE"
        echo "✔ Added git diff."
    fi
fi

# --- Finalise (Wayland) ---
if [ -s "$TMP_FILE" ]; then
    wl-copy < "$TMP_FILE"
    echo -e "\n✅ Success! Context copied to clipboard."
else
    echo -e "\n⚠️ No content generated. Clipboard preserved."
fi

rm "$TMP_FILE" 2>/dev/null