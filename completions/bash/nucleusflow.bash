# bash completion for nucleusflow                        -*- shell-script -*-
# SPDX-License-Identifier: Apache-2.0 OR MIT

_nucleusflow() {
    local cur prev words cword
    _init_completion || return

    local commands="new build serve help"

    if [[ $cword -eq 1 ]]; then
        COMPREPLY=( $(compgen -W "$commands -v --verbose -h --help -V --version" -- "$cur") )
        return 0
    fi

    case "${words[1]}" in
        new)
            case "$prev" in
                -t|--template)
                    COMPREPLY=( $(compgen -W "blog docs portfolio" -- "$cur") )
                    return 0
                    ;;
                *)
                    COMPREPLY=( $(compgen -W "-t --template -h --help" -- "$cur") )
                    return 0
                    ;;
            esac
            ;;
        build)
            case "$prev" in
                -c|--content-dir|-o|--output-dir|-t|--template-dir|-f|--config)
                    _filedir
                    return 0
                    ;;
                *)
                    COMPREPLY=( $(compgen -W "-c --content-dir -o --output-dir -t --template-dir -m --minify -f --config -h --help" -- "$cur") )
                    return 0
                    ;;
            esac
            ;;
        serve)
            case "$prev" in
                -d|--dir)
                    _filedir -d
                    return 0
                    ;;
                -p|--port)
                    return 0
                    ;;
                *)
                    COMPREPLY=( $(compgen -W "-p --port -w --watch -d --dir -h --help" -- "$cur") )
                    return 0
                    ;;
            esac
            ;;
    esac
}

complete -F _nucleusflow nucleusflow
