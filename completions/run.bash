#!/usr/bin/env bash
# Bash completion script for run command

_run_complete() {
    local cur prev opts
    COMPREPLY=()
    cur="${COMP_WORDS[COMP_CWORD]}"
    prev="${COMP_WORDS[COMP_CWORD-1]}"

    # Basic options
    opts="--list --generate-completion --version --help -l -h"

    # If we're completing after --generate-completion, suggest shells
    if [[ "${prev}" == "--generate-completion" ]]; then
        COMPREPLY=( $(compgen -W "bash zsh fish" -- "${cur}") )
        return 0
    fi

    # If the previous word is a flag, let normal completion happen
    if [[ "${prev}" == -* ]]; then
        return 0
    fi

    # First argument: show top-level commands and flags
    if [[ ${COMP_CWORD} -eq 1 ]]; then
        # Use cached functions if available and recent (< 5 seconds old)
        local cache_key="${PWD}"
        local cache_hash=""
        if command -v md5sum >/dev/null 2>&1; then
            cache_hash=$(echo "$cache_key" | md5sum | cut -d' ' -f1)
        elif command -v md5 >/dev/null 2>&1; then
            cache_hash=$(echo "$cache_key" | md5 | awk '{print $NF}')
        elif command -v shasum >/dev/null 2>&1; then
            cache_hash=$(echo "$cache_key" | shasum | cut -d' ' -f1)
        else
            # Fallback: use sanitized path (not cryptographically strong, but unique-ish)
            cache_hash=$(echo "$cache_key" | tr '/\\' '__')
        fi
        local cache_file="${TMPDIR:-/tmp}/.run_completion_cache_${USER}_${cache_hash}"
        local completions=""

        # Check if cache exists and is less than 5 seconds old
        if [[ -f "$cache_file" ]] && find "$cache_file" -type f -mtime -0.00006 >/dev/null 2>&1; then
            completions=$(cat "$cache_file" 2>/dev/null)
        else
            # Get functions and extract top-level names
            if command -v run &> /dev/null; then
                local all_funcs=$(run --list 2>/dev/null | sed -n 's/^  //p')
                local top_level=""
                local -A seen

                while IFS= read -r func; do
                    if [[ $func == *:* ]]; then
                        # Extract prefix before colon
                        local prefix="${func%%:*}"
                        if [[ -z "${seen[$prefix]}" ]]; then
                            top_level="${top_level}${prefix} "
                            seen[$prefix]=1
                        fi
                    else
                        # Non-nested function
                        top_level="${top_level}${func} "
                    fi
                done <<< "$all_funcs"

                completions="$top_level"
                echo "$completions" > "$cache_file" 2>/dev/null
            fi
        fi

        # Combine options and top-level commands
        local all_completions="${opts} ${completions}"

        COMPREPLY=( $(compgen -W "${all_completions}" -- "${cur}") )

    # Second argument: if prev is a namespace, show subcommands
    elif [[ ${COMP_CWORD} -eq 2 ]]; then
        local namespace="$prev"

        if command -v run &> /dev/null; then
            local all_funcs=$(run --list 2>/dev/null | sed -n 's/^  //p')
            local subcommands=""

            while IFS= read -r func; do
                if [[ $func == ${namespace}:* ]]; then
                    # Extract part after colon
                    local subcmd="${func#*:}"
                    subcommands="${subcommands}${subcmd} "
                fi
            done <<< "$all_funcs"

            if [[ -n "$subcommands" ]]; then
                COMPREPLY=( $(compgen -W "${subcommands}" -- "${cur}") )
            fi
        fi
    fi

    return 0
}

complete -F _run_complete run
