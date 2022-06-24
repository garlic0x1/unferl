# unferl  
A mostly backwards compatible clone of https://github.com/tomnomnom/unfurl in Rust.  
  
```
$ unferl -h
unferl 0.1.0

USAGE:
    unferl [OPTIONS] <SUBCOMMAND>

OPTIONS:
    -h, --help       Print help information
    -u, --unique     only print unique results
    -v, --verbose    print errors to stderr
    -V, --version    Print version information

SUBCOMMANDS:
    domains      print domains
    format       printf style formatting (`unferl help format`)
    fragments    print fragments
    help         Print this message or the help of the given subcommand(s)
    keypairs     print keypairs ("key=value")
    keys         print query keys
    paths        print paths
    values       print query values

``` 
# formatting  
```
$ unferl help format
...
printf style formatting
%% -> literal "%"
%s -> scheme
%u -> user info (user:pass)
%a -> authority (alias for %u%@%d%:%P)
%d -> domain
%S -> subdomain
%r -> root domain
%t -> TLD (".com", ".org", etc)
%P -> port
%p -> path
%q -> query string
%f -> fragment
%@ -> insert @ if user info specified
%: -> insert : if port specified
%? -> insert ? if query exists
%# -> insert # if fragment exists
...
```
