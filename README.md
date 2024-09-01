```sh
monitor stdin/stdout/stderr

Usage: stdio-monitor [OPTIONS] -- <COMMAND>...

Arguments:
  <COMMAND>...  command to execute the program with arguments...

Options:
      --stdin <STDIN>    Path to log stdin traffic; default to stderr
      --stdout <STDOUT>  Path to log stdout traffic; default to stderr
      --stderr <STDERR>  Path to log stderr traffic; default to stderr
  -h, --help             Print help
```
