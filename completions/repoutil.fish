# Print an optspec for argparse to handle cmd's options that are independent of any subcommand.
function __fish_repoutil_global_optspecs
	string join \n j/json= color= threads= k/keep-home= h/help
end

function __fish_repoutil_needs_command
	# Figure out if the current invocation already has a command.
	set -l cmd (commandline -opc)
	set -e cmd[1]
	argparse -s (__fish_repoutil_global_optspecs) -- $cmd 2>/dev/null
	or return
	if set -q argv[1]
		# Also print the command, so this can be used to figure out what it is.
		echo $argv[1]
		return 1
	end
	return 0
end

function __fish_repoutil_using_subcommand
	set -l cmd (__fish_repoutil_needs_command)
	test -z "$cmd"
	and return 1
	contains -- $cmd[1] $argv
end

complete -c repoutil -n "__fish_repoutil_needs_command" -s j -l json -d 'Output as JSON' -r
complete -c repoutil -n "__fish_repoutil_needs_command" -l color -d 'Colorize output' -r -f -a "auto\t''
always\t''
never\t''"
complete -c repoutil -n "__fish_repoutil_needs_command" -l threads -d 'Limit thread pool size' -r
complete -c repoutil -n "__fish_repoutil_needs_command" -s k -l keep-home -r
complete -c repoutil -n "__fish_repoutil_needs_command" -s h -l help -d 'Print help'
complete -c repoutil -n "__fish_repoutil_needs_command" -f -a "list" -d 'List directories tracked in ~/.repoutilrc'
complete -c repoutil -n "__fish_repoutil_needs_command" -f -a "add" -d 'Add current directory to ~/.repoutilrc'
complete -c repoutil -n "__fish_repoutil_needs_command" -f -a "git" -d 'Operations on git repositories'
complete -c repoutil -n "__fish_repoutil_needs_command" -f -a "jj" -d 'Operations on git repositories'
complete -c repoutil -n "__fish_repoutil_needs_command" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c repoutil -n "__fish_repoutil_using_subcommand list" -s h -l help -d 'Print help'
complete -c repoutil -n "__fish_repoutil_using_subcommand add" -s h -l help -d 'Print help'
complete -c repoutil -n "__fish_repoutil_using_subcommand git; and not __fish_seen_subcommand_from stat fetch pull push branches branchstat stashcount unclean dashboard untracked help" -s h -l help -d 'Print help'
complete -c repoutil -n "__fish_repoutil_using_subcommand git; and not __fish_seen_subcommand_from stat fetch pull push branches branchstat stashcount unclean dashboard untracked help" -f -a "stat" -d 'Show short status'
complete -c repoutil -n "__fish_repoutil_using_subcommand git; and not __fish_seen_subcommand_from stat fetch pull push branches branchstat stashcount unclean dashboard untracked help" -f -a "fetch" -d 'Fetch commits and tags'
complete -c repoutil -n "__fish_repoutil_using_subcommand git; and not __fish_seen_subcommand_from stat fetch pull push branches branchstat stashcount unclean dashboard untracked help" -f -a "pull" -d 'Pull commits'
complete -c repoutil -n "__fish_repoutil_using_subcommand git; and not __fish_seen_subcommand_from stat fetch pull push branches branchstat stashcount unclean dashboard untracked help" -f -a "push" -d 'Push commits'
complete -c repoutil -n "__fish_repoutil_using_subcommand git; and not __fish_seen_subcommand_from stat fetch pull push branches branchstat stashcount unclean dashboard untracked help" -f -a "branches" -d 'List all branches'
complete -c repoutil -n "__fish_repoutil_using_subcommand git; and not __fish_seen_subcommand_from stat fetch pull push branches branchstat stashcount unclean dashboard untracked help" -f -a "branchstat" -d 'List short status of all branches'
complete -c repoutil -n "__fish_repoutil_using_subcommand git; and not __fish_seen_subcommand_from stat fetch pull push branches branchstat stashcount unclean dashboard untracked help" -f -a "stashcount" -d 'Count stashes'
complete -c repoutil -n "__fish_repoutil_using_subcommand git; and not __fish_seen_subcommand_from stat fetch pull push branches branchstat stashcount unclean dashboard untracked help" -f -a "unclean" -d 'List repos with local changes'
complete -c repoutil -n "__fish_repoutil_using_subcommand git; and not __fish_seen_subcommand_from stat fetch pull push branches branchstat stashcount unclean dashboard untracked help" -f -a "dashboard" -d 'Display git dashboard'
complete -c repoutil -n "__fish_repoutil_using_subcommand git; and not __fish_seen_subcommand_from stat fetch pull push branches branchstat stashcount unclean dashboard untracked help" -f -a "untracked" -d 'List all untracked folders'
complete -c repoutil -n "__fish_repoutil_using_subcommand git; and not __fish_seen_subcommand_from stat fetch pull push branches branchstat stashcount unclean dashboard untracked help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c repoutil -n "__fish_repoutil_using_subcommand git; and __fish_seen_subcommand_from stat" -s h -l help -d 'Print help'
complete -c repoutil -n "__fish_repoutil_using_subcommand git; and __fish_seen_subcommand_from fetch" -s h -l help -d 'Print help'
complete -c repoutil -n "__fish_repoutil_using_subcommand git; and __fish_seen_subcommand_from pull" -s h -l help -d 'Print help'
complete -c repoutil -n "__fish_repoutil_using_subcommand git; and __fish_seen_subcommand_from push" -s h -l help -d 'Print help'
complete -c repoutil -n "__fish_repoutil_using_subcommand git; and __fish_seen_subcommand_from branches" -s h -l help -d 'Print help'
complete -c repoutil -n "__fish_repoutil_using_subcommand git; and __fish_seen_subcommand_from branchstat" -s h -l help -d 'Print help'
complete -c repoutil -n "__fish_repoutil_using_subcommand git; and __fish_seen_subcommand_from stashcount" -s h -l help -d 'Print help'
complete -c repoutil -n "__fish_repoutil_using_subcommand git; and __fish_seen_subcommand_from unclean" -s h -l help -d 'Print help'
complete -c repoutil -n "__fish_repoutil_using_subcommand git; and __fish_seen_subcommand_from dashboard" -s h -l help -d 'Print help'
complete -c repoutil -n "__fish_repoutil_using_subcommand git; and __fish_seen_subcommand_from untracked" -s h -l help -d 'Print help'
complete -c repoutil -n "__fish_repoutil_using_subcommand git; and __fish_seen_subcommand_from help" -f -a "stat" -d 'Show short status'
complete -c repoutil -n "__fish_repoutil_using_subcommand git; and __fish_seen_subcommand_from help" -f -a "fetch" -d 'Fetch commits and tags'
complete -c repoutil -n "__fish_repoutil_using_subcommand git; and __fish_seen_subcommand_from help" -f -a "pull" -d 'Pull commits'
complete -c repoutil -n "__fish_repoutil_using_subcommand git; and __fish_seen_subcommand_from help" -f -a "push" -d 'Push commits'
complete -c repoutil -n "__fish_repoutil_using_subcommand git; and __fish_seen_subcommand_from help" -f -a "branches" -d 'List all branches'
complete -c repoutil -n "__fish_repoutil_using_subcommand git; and __fish_seen_subcommand_from help" -f -a "branchstat" -d 'List short status of all branches'
complete -c repoutil -n "__fish_repoutil_using_subcommand git; and __fish_seen_subcommand_from help" -f -a "stashcount" -d 'Count stashes'
complete -c repoutil -n "__fish_repoutil_using_subcommand git; and __fish_seen_subcommand_from help" -f -a "unclean" -d 'List repos with local changes'
complete -c repoutil -n "__fish_repoutil_using_subcommand git; and __fish_seen_subcommand_from help" -f -a "dashboard" -d 'Display git dashboard'
complete -c repoutil -n "__fish_repoutil_using_subcommand git; and __fish_seen_subcommand_from help" -f -a "untracked" -d 'List all untracked folders'
complete -c repoutil -n "__fish_repoutil_using_subcommand git; and __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c repoutil -n "__fish_repoutil_using_subcommand jj; and not __fish_seen_subcommand_from stat sync help" -s h -l help -d 'Print help'
complete -c repoutil -n "__fish_repoutil_using_subcommand jj; and not __fish_seen_subcommand_from stat sync help" -f -a "stat" -d 'Get status of all repositories'
complete -c repoutil -n "__fish_repoutil_using_subcommand jj; and not __fish_seen_subcommand_from stat sync help" -f -a "sync" -d 'Pull all repositories'
complete -c repoutil -n "__fish_repoutil_using_subcommand jj; and not __fish_seen_subcommand_from stat sync help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c repoutil -n "__fish_repoutil_using_subcommand jj; and __fish_seen_subcommand_from stat" -s h -l help -d 'Print help'
complete -c repoutil -n "__fish_repoutil_using_subcommand jj; and __fish_seen_subcommand_from sync" -s h -l help -d 'Print help'
complete -c repoutil -n "__fish_repoutil_using_subcommand jj; and __fish_seen_subcommand_from help" -f -a "stat" -d 'Get status of all repositories'
complete -c repoutil -n "__fish_repoutil_using_subcommand jj; and __fish_seen_subcommand_from help" -f -a "sync" -d 'Pull all repositories'
complete -c repoutil -n "__fish_repoutil_using_subcommand jj; and __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c repoutil -n "__fish_repoutil_using_subcommand help; and not __fish_seen_subcommand_from list add git jj help" -f -a "list" -d 'List directories tracked in ~/.repoutilrc'
complete -c repoutil -n "__fish_repoutil_using_subcommand help; and not __fish_seen_subcommand_from list add git jj help" -f -a "add" -d 'Add current directory to ~/.repoutilrc'
complete -c repoutil -n "__fish_repoutil_using_subcommand help; and not __fish_seen_subcommand_from list add git jj help" -f -a "git" -d 'Operations on git repositories'
complete -c repoutil -n "__fish_repoutil_using_subcommand help; and not __fish_seen_subcommand_from list add git jj help" -f -a "jj" -d 'Operations on git repositories'
complete -c repoutil -n "__fish_repoutil_using_subcommand help; and not __fish_seen_subcommand_from list add git jj help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c repoutil -n "__fish_repoutil_using_subcommand help; and __fish_seen_subcommand_from git" -f -a "stat" -d 'Show short status'
complete -c repoutil -n "__fish_repoutil_using_subcommand help; and __fish_seen_subcommand_from git" -f -a "fetch" -d 'Fetch commits and tags'
complete -c repoutil -n "__fish_repoutil_using_subcommand help; and __fish_seen_subcommand_from git" -f -a "pull" -d 'Pull commits'
complete -c repoutil -n "__fish_repoutil_using_subcommand help; and __fish_seen_subcommand_from git" -f -a "push" -d 'Push commits'
complete -c repoutil -n "__fish_repoutil_using_subcommand help; and __fish_seen_subcommand_from git" -f -a "branches" -d 'List all branches'
complete -c repoutil -n "__fish_repoutil_using_subcommand help; and __fish_seen_subcommand_from git" -f -a "branchstat" -d 'List short status of all branches'
complete -c repoutil -n "__fish_repoutil_using_subcommand help; and __fish_seen_subcommand_from git" -f -a "stashcount" -d 'Count stashes'
complete -c repoutil -n "__fish_repoutil_using_subcommand help; and __fish_seen_subcommand_from git" -f -a "unclean" -d 'List repos with local changes'
complete -c repoutil -n "__fish_repoutil_using_subcommand help; and __fish_seen_subcommand_from git" -f -a "dashboard" -d 'Display git dashboard'
complete -c repoutil -n "__fish_repoutil_using_subcommand help; and __fish_seen_subcommand_from git" -f -a "untracked" -d 'List all untracked folders'
complete -c repoutil -n "__fish_repoutil_using_subcommand help; and __fish_seen_subcommand_from jj" -f -a "stat" -d 'Get status of all repositories'
complete -c repoutil -n "__fish_repoutil_using_subcommand help; and __fish_seen_subcommand_from jj" -f -a "sync" -d 'Pull all repositories'
