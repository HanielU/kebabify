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

Show help information:

```bash
kebabify --help
```

Basic usage (current directory):

```bash
kebabify
```

Specify a directory:

```bash
kebabify /path/to/directory
```

Process import statements instead of filenames:

```bash
kebabify -i /path/to/directory
```

Process both filenames and import statements:

```bash
kebabify -a /path/to/directory
```

### Examples

Before:

```
MyProject/
├── ComponentLibrary/
│ ├── ButtonComponent.svelte
│ └── InputField.svelte
└── UtilityFunctions.ts
```

After:

```
my-project/
├── component-library/
│ ├── button-component.svelte
│ └── input-field.svelte
└── utility-functions.ts
```

Import statement conversion:

Before:

```
import ButtonComponent from './ComponentLibrary/ButtonComponent.svelte';
const Utils = require('./UtilityFunctions');
```

After:

```
import ButtonComponent from './component-library/button-component.svelte';
const Utils = require('./utility-functions');
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
