# unferl  
A clone of https://github.com/tomnomnom/unfurl in Rust.  
Format string functionality works the same, but without all fields implemented yet.  
  
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
    domains     print domains
    format      printf style formatting (`unferl help format` for more details)
    help        Print this message or the help of the given subcommand(s)
    keypairs    print keypairs ("key=value")
    keys        print query keys
    paths       print paths
    values      print query values

``` 

