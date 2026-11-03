# Seal

<img src="docs/img/seal-logo-horizontal.svg" alt="seal" width="600">

A hobby shell written in Rust, scriptable with Lua.

> ⚠️ Seal is under very agile development. Expect breaking changes.

Seal is my hobby project and my daily-driver shell. It covers all my own requirements, and I'm happy with it.

## Features

- **Lua builtins.** Write functions in Lua, register them with the shell runtime, and run them like any other builtin command. (This is similar to aliases in bash, but I haven't explored the full potential of the feature yet.)
- **Plug-and-play Rust builtins.** A builtin is just a struct implementing a Rust trait. Define it however you want, as long as it implements the trait.

## Philosophy

Seal is a niche shell, and it makes choices that not everyone will like:

- It does **not** fully follow POSIX conventions.
- It is built around Lua scripting rather than bash-style scripting.
- I follow a different ideology form normal people so seal will 100% refelect it.

## Limitations

Please read these before trying seal:

- **Not POSIX-compliant.** Seal cannot run bash scripts on its own.
- **Interactive only.** It cannot be used as a proper login shell yet.
- **Unstable.** Config format, APIs, and behaviour may change between commits.

## Installation

You have two options.

### Compile from source (recommended)

Release binaries may lag behind the latest code, so I recommend building from source for now.

```sh
git clone https://github.com/4rjxn/seal.git
cd seal
cargo build --release
```

The binary will be at `target/release/seal`.

### Release binary

Download a binary from the [releases page](https://github.com/<user>/seal/releases). Releases are more tested, but they lack the newest features.

## Configuration

Seal reads its Lua config from `~/.config/seal/init.lua`. Create the file if it doesn't exist.

## Lua API

Seal currently exposes two functions, and nothing more:

| Function | Description |
| --- | --- |
| `seal.run(...)` | Returns the exit code of the process, run takes in a raw string (eg: "cat help.txt") |
| `seal.register(name,discription, fn)` | Registers a Lua function as a builtin command named `name`. |

## Writing a Lua builtin

1. Create `~/.config/seal/init.lua` if it does not exist.
2. Write a function in that file.
3. Register it with `seal.register()`.
4. Restart the shell.

```lua
-- ~/.config/seal/init.lua
seal.register("ll", function(args)
	seal.run("ls -lah --color")
end)
```

Then, in seal:

```sh
$ ll
<see for yourself :)>
```

## Writing a Rust builtin

1. Create a new file named after your builtin under `src/shell/builtins/`.
2. Define a struct in that file.
3. Implement the `BuiltIn` trait for that struct.
4. Add the file as a module in `src/shell/builtins.rs`.
5. Register the struct in the builtin registry.


## Contributing

always welcome.

