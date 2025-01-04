# kebabify

A command-line tool that recursively converts PascalCase filenames to kebab-case in a directory.

## Features

- Recursively processes all files and directories
- Handles nested directories correctly (bottom-up approach)
- Preserves file extensions
- Safe handling of special characters and paths
- Follows symbolic links

## Installation (from source)

```bash
cargo install --path .
```

## Usage

Basic usage (current directory):

```bash
kebabify
```

Specify a directory:

```bash
kebabify /path/to/directory
```

### Examples

Before:

```
MyProject/
├── ComponentLibrary/
│   ├── ButtonComponent.svelte
│   └── InputField.svelte
└── UtilityFunctions.ts
```

After:

```
my-project/
├── component-library/
│   ├── button-component.svelte
│   └── input-field.svelte
└── utility-functions.ts
```

## Building from Source

1. Clone the repository
2. Build the project:

```bash
cargo build --release
```

3. The binary will be available in `target/release/kebabify`

## License

MIT License

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.
