# Print an optspec for argparse to handle cmd's options that are independent of any subcommand.
function __fish_toss_global_optspecs
    string join \n v h/help V/version
end

function __fish_toss_needs_command
    # Figure out if the current invocation already has a command.
    set -l cmd (commandline -opc)
    set -e cmd[1]
    argparse -s (__fish_toss_global_optspecs) -- $cmd 2>/dev/null
    or return
    if set -q argv[1]
        # Also print the command, so this can be used to figure out what it is.
        echo $argv[1]
        return 1
    end
    return 0
end

function __fish_toss_using_subcommand
    set -l cmd (__fish_toss_needs_command)
    test -z "$cmd"
    and return 1
    contains -- $cmd[1] $argv
end

complete -c toss -n "__fish_toss_needs_command" -s v -d 'Print version information'
complete -c toss -n "__fish_toss_needs_command" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c toss -n "__fish_toss_needs_command" -s V -l version -d 'Print version'
complete -c toss -n "__fish_toss_needs_command" -f -a "put" -d 'Move files or directories to the trash'
complete -c toss -n "__fish_toss_needs_command" -f -a "list" -d 'Browse trashed files in a TUI'
complete -c toss -n "__fish_toss_needs_command" -f -a "restore" -d 'Interactively restore trashed files via TUI'
complete -c toss -n "__fish_toss_needs_command" -f -a "empty" -d 'Empty the trash (optionally only files older than N days)'
complete -c toss -n "__fish_toss_needs_command" -f -a "rm" -d 'Remove trashed files matching a glob pattern'
complete -c toss -n "__fish_toss_needs_command" -f -a "completions" -d 'Generate shell autocompletions (bash, zsh, fish, powershell, elvish)'
complete -c toss -n "__fish_toss_needs_command" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c toss -n "__fish_toss_using_subcommand put" -s h -l help -d 'Print help'
complete -c toss -n "__fish_toss_using_subcommand list" -s h -l help -d 'Print help'
complete -c toss -n "__fish_toss_using_subcommand restore" -l overwrite -d 'Overwrite existing files at original path'
complete -c toss -n "__fish_toss_using_subcommand restore" -s h -l help -d 'Print help'
complete -c toss -n "__fish_toss_using_subcommand empty" -s h -l help -d 'Print help'
complete -c toss -n "__fish_toss_using_subcommand rm" -s h -l help -d 'Print help'
complete -c toss -n "__fish_toss_using_subcommand completions" -s h -l help -d 'Print help'
complete -c toss -n "__fish_toss_using_subcommand help; and not __fish_seen_subcommand_from put list restore empty rm completions help" -f -a "put" -d 'Move files or directories to the trash'
complete -c toss -n "__fish_toss_using_subcommand help; and not __fish_seen_subcommand_from put list restore empty rm completions help" -f -a "list" -d 'Browse trashed files in a TUI'
complete -c toss -n "__fish_toss_using_subcommand help; and not __fish_seen_subcommand_from put list restore empty rm completions help" -f -a "restore" -d 'Interactively restore trashed files via TUI'
complete -c toss -n "__fish_toss_using_subcommand help; and not __fish_seen_subcommand_from put list restore empty rm completions help" -f -a "empty" -d 'Empty the trash (optionally only files older than N days)'
complete -c toss -n "__fish_toss_using_subcommand help; and not __fish_seen_subcommand_from put list restore empty rm completions help" -f -a "rm" -d 'Remove trashed files matching a glob pattern'
complete -c toss -n "__fish_toss_using_subcommand help; and not __fish_seen_subcommand_from put list restore empty rm completions help" -f -a "completions" -d 'Generate shell autocompletions (bash, zsh, fish, powershell, elvish)'
complete -c toss -n "__fish_toss_using_subcommand help; and not __fish_seen_subcommand_from put list restore empty rm completions help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
