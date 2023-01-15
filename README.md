# Run-on-change

Run command on files changed.

# Usage

```
run-on-change <PATTERN> <COMMAND> [[--] COMMAND_ARGS]...
```
For example, build automatically on java files changed:
```
run-on-change "*.java" sh -- gradlew clean build
```

More usage please run: `run-on-change --help`

# Supported platform

* Linux
* Windows
* MacOS

# Licence

MIT