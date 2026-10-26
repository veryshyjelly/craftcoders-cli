# CraftCoders Local CLI

A high-performance Rust implementation of a local CodeCrafters-compatible CLI tool for running coding challenges offline. This tool enables you to practice programming challenges like Build Your Own Redis, Git, SQLite, and more entirely on your local machine without requiring internet access or the CodeCrafters website.

## ✨ Features

- **Blazing Fast**: Built in Rust for minimal startup time and efficient execution
- **Complete Offline Workflow**: Run challenges entirely offline with zero external dependencies
- **Smart Progress Tracking**: Automatically tracks completion status across all stages
- **Official Tester Integration**: Runs the actual challenge testers locally against your code
- **Extension System**: Seamlessly add community-created extensions to your challenges
- **Multi-Language Support**: Works with any programming language via your `your_program.sh` script
- **Docker Integration**: Optional containerized testing for isolated environments
- **Rich Terminal Output**: Colorized stage headers, progress indicators, and clear test results
- **Extensible Design**: Modular architecture making it easy to add new features

## 🚀 Installation

### Prerequisites
- Rust 1.65+ (for building from source)
- Go 1.19+ (for building testers) OR Docker (for containerized testing)
- Git (for cloning challenge repositories)

### From Source (Recommended)
```bash
# Clone the repository
git clone https://github.com/veryshyjelly/craftcoders-cli.git
cd craftcoders-cli

# Build and install
cargo install --path .

# Verify installation
craftcoders --version
# CraftCoders Local CLI v0.1.0
```

## 🚦 Quick Start

### 1. Initialize a Challenge
```bash
# Initialize a Redis challenge in a directory named 'my-redis'
craftcoders init redis my-redis
craftcoders my-redis
```

### 2. View Stage Instructions
```bash
# Show current stage instructions
craftcoders show

# Show instructions for a specific stage
craftcoders show 3

# Show instructions without ANSI colors (for piping to files)
craftcoders show --no-color
```

### 3. Implement Your Solution
Edit `your_program.sh` to launch your solution:
```bash
#!/usr/bin/env sh
# Example for a Python solution
python3 redis_server.py "$@"

# Example for a Rust solution
./target/debug/redis_impl "$@"

# Example for a JavaScript solution
node redis_impl.js "$@"
```

Make sure your script is executable:
```bash
chmod +x your_program.sh
```

### 4. Test Your Progress
```bash
# Test only the current stage without advancing
craftcoders test

# Test all stages through the current stage and advance if successful
craftcoders submit

# Explicitly test all stages through a given stage (number or slug)
craftcoders test --upto 3

# Run with Docker (optional)
craftcoders test --docker redis:7-alpine
```

### 5. Track Your Progress
```bash
# Show detailed progress
craftcoders status

# Show just the current stage number
craftcoders status --current-only
```

## 🏗️ How It Works

### Architecture Overview
```
┌─────────────────────┐    ┌──────────────────────┐    ┌────────────────────┐
│ CraftCoders Local   │    │ Challenge Repository │    │ Tester Binary      │
│        CLI          │◄──▶│ (course content +    │◄──▶│ (Go/Rust binary)   │
│                     │    │  test cases)         │    │                    │
└─────────────────────┘    └──────────────────────┘    └────────────────────┘
          │                         ▲                         │
          │                         │                         │
          ▼                         │                         │
┌─────────────────────┐             │                         │
│  Your Solution      │             │                         │
│ (your_program.sh)   │─────────────┘                         │
│                     │                                       │
└─────────────────────┘                                       │
          │                                                   │
          ▼                                                   │
┌─────────────────────┐                             ┌──────────────────┐
│ Test Execution      │◄────────────────────────────┤ Test Results     │
│ (in isolated env)   │                             │ (pass/fail, logs)│
└─────────────────────┘                             └──────────────────┘
```

### Progress Tracking System
- Stores completion status in the project's `.craftcoders.json`
- Tests only the current stage with `craftcoders test`, without changing progress
- Re-tests all stages through the current stage with `craftcoders submit` and advances only when they all pass
- Supports resuming progress after interruptions

## 📖 Command Reference

### Initialization & Setup
| Command                                    | Description                                 |
| ------------------------------------------ | ------------------------------------------- |
| `craftcoders init <challenge> <directory>` | Initialize a new challenge workspace        |
| `craftcoders update`                       | Update challenge content and rebuild tester |
| `craftcoders doctor`                       | Check environment for common issues         |

### Core Workflow
| Command                                 | Description                                         |
| --------------------------------------- | --------------------------------------------------- |
| `craftcoders show [stage] [--no-color]` | Display instructions for current or specified stage |
| `craftcoders test [--docker <image>]`   | Run tests for only the current stage                |
| `craftcoders submit [--docker <image>]` | Test all stages through current and advance on pass |
| `craftcoders status [--current-only]`   | Show progress through the challenge                 |

### Extension Management
| Command                                        | Description                           |
| ---------------------------------------------- | ------------------------------------- |
| `craftcoders ext add <name> [--url <git-url>]` | Add an extension from Git repository  |
| `craftcoders ext list`                         | List installed extensions             |
| `craftcoders ext remove <name>`                | Remove an installed extension         |
| `craftcoders ext update <name>`                | Update an extension to latest version |

### Utility Commands
| Command                                         | Description                            |
| ----------------------------------------------- | -------------------------------------- |
| `craftcoders completion <shell>`                         | Generate shell completion script       |
| `craftcoders config [--get <key>] [--set <key>=<value>]` | Manage CLI configuration               |
| `craftcoders cleanup`                                    | Remove cached data and temporary files |

## 🛠️ Configuration

### Global Settings (`~/.craftcoders/config.toml`)
```toml
# Default tester build method when Go is unavailable
tester_builder = "docker"  # or "go"

# Default Docker image for tester builds
default_tester_image = "golang:1.19-alpine"

# Enable/disable color output
color = true

# Progress tracking settings
auto_save_progress = true
progress_file = "/home/user/.craftcoders/progress.json"
```

### Workspace State (`./.craftcoders.json`)
```json
{
  "course": "redis",
  "extensions": [],
  "completed": [
    "jm1"
  ]
}
```

## 💡 Advanced Usage

### Custom Test Environments
```bash
# Use a specific Docker image for testing
craftcoders test --docker my-custom-redis-tester:latest

# Build tester with specific Go flags
CC_TESTER_BUILD_FLAGS="-ldflags='-s -w'" craftcoders update

# Skip tester rebuild (use existing binary)
craftcoders update --skip-tester-build
```

### Working with Extensions
```bash
# Add a community extension
craftcoders ext add redis-aof https://github.com/username/redis-aof-extension

# List all active extensions
craftcoders ext list

# Update all extensions
craftcoders ext update --all

# Remove an extension
craftcoders ext remove redis-aof
```

### Automation & Scripting
```bash
# Check if current stage is complete in shell scripts
if craftcoders status --current-only | grep -q "done"; then
  echo "Stage is complete!"
fi

# Get just the stage number for scripting
craftcoders status --current-only --format=json | jq '.current_stage'

# Run tests and capture JUnit-compatible output
craftcoders test --format=junit > test-results.xml
```

## 🔧 Development

### Building from Source
```bash
# Debug build (with symbols)
cargo build

# Release build (optimized)
cargo build --release

# Cross-compile for Linux (from macOS/Windows)
cargo build --release --target x86_64-unknown-linux-gnu

# Create a distributable tarball
cargo package
```

### Testing
```bash
# Run unit tests
cargo test

# Run tests with coverage
cargo tarpaulin --out Html

# Run specific test suite
cargo test --test integration_test

# Run tests in release mode (closer to production behavior)
cargo test --release
```

### Adding New Features
1. Fork the repository
2. Create a feature branch: `git checkout -b feature/awesome-feature`
3. Make your changes following the existing code style
4. Add tests for new functionality
5. Ensure all tests pass: `cargo test --release`
6. Submit a Pull Request with a clear description

### Code Structure
```
src/
├── main.rs          # CLI entry point and command dispatch
├── render.rs        # Template resolution & markdown rendering
├── course.rs        # Course definition parsing & handling
├── tester.rs        # Tester management & execution logic
├── progress.rs      # Progress tracking & persistence
├── utils.rs         # Helper functions & constants
├── commands/        # Individual command implementations
│   ├── init.rs
│   ├── show.rs
│   ├── test.rs
│   ├── submit.rs
│   └── status.rs
└── extensions/      # Extension system implementation
```

## 🐛 Troubleshooting

### Common Issues

| Problem                              | Solution                                                     |
| ------------------------------------ | ---------------------------------------------------------    |
| `command not found: craftcoders`     | Ensure `~/.cargo/bin` is in your PATH                        |
| Testers fail to build                | Install Go 1.19+ or ensure Docker is running                 |
| Permission denied on your_program.sh | Run `chmod +x your_program.sh`                               |
| Docker permission errors             | Add user to docker group: `sudo usermod -aG docker $USER`    |
| Stuck on "Downloading test cases..." | Check internet connectivity or use `--offline` flag          |

### Debugging
```bash
# Enable verbose logging
RUST_LOG=debug craftcoders test

# Log to file
RUST_LOG=info craftcoders test 2> craftcoders.log
```

## 📚 Related Projects

- [CodeCrafters Official](https://codecrafters.io/) - The original platform
- [Build Your Own Redis](https://github.com/codecrafters-io/build-your-own-redis) - Redis challenge repo
- [Build Your Own Git](https://github.com/codecrafters-io/build-your-own-git) - Git challenge repo
- [Tester Utils](https://github.com/codecrafters-io/tester-utils) - Shared tester library

## 🙏 Acknowledgments

- Thanks to the [CodeCrafters team](https://codecrafters.io/) for creating such an excellent learning platform
- The Rust community for providing amazing tools and libraries
- Open-source contributors who maintain the challenge and tester repositories

## 📄 License

This project is licensed under the [MIT License](LICENSE).

## 📢 Support & Feedback

If you enjoy using CraftCoders Local CLI, please consider:

1. ⭐ Starring the repository on GitHub
2. � Reporting bugs or suggesting features via [Issues](https://github.com/veryshyjelly/craftcoders-cli/issues)
3. 💬 Sharing your experience on social media with #CraftCodersLocal
4. 🤝 Contributing code, documentation, or translations

Happy coding, and may your tests always pass! 🎉

---

*Built with ❤️ using Rust. Not affiliated with or endorsed by CodeCrafters.io.*
