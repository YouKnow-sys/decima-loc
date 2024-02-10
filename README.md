<h1 align="center">Decima Localization Manager</h1>

<p align="center">
  <b>Localization management tool for games built on the Decima engine</b></br>
  <sub>Exports and imports game text in various formats for easy localization</sub>
</p>

<div align="center">
  
[![Build Status](https://github.com/YouKnow-sys/decima-loc/actions/workflows/rust.yml/badge.svg)](https://github.com/YouKnow-sys/decima-loc/actions?workflow=Rust%20CI)
[![unsafe forbidden](https://img.shields.io/badge/unsafe-forbidden-success.svg)](https://github.com/rust-secure-code/safety-dance/)
[![MIT](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/YouKnow-sys/ar-reshaper/blob/master/LICENSE)

</div>

<p align="center">
  <a href="#features">Features</a> •
  <a href="#supported-games">Supported Games</a> •
  <a href="#installation">Installation</a> •
  <a href="#usage">Usage</a> •
  <a href="#contributing">Contributing</a> •
  <a href="#license">License</a>
</p>

## Features

- Export and import game text in:
  - Plain text (TXT)
  - JSON
  - YAML
- Batch export/import for multiple files
- CLI and library interfaces
- Support to export all or part of languages
- Easy to use interface for non-technical users

## Supported Games 

| Game                     | Status        |
|--------------------------|---------------|
| Horizon Zero Dawn        | Supported ✅  |
| Death Stranding          | Supported ✅  |

## Installation

You can download latest version of the program from **Release** page

## Usage

Export text from Horizon Zero Dawn to JSON:

```
decima-loc hzd single "path-to-core" export --format json
```

Import translated text back into the game and create new core file: 

```
decima-loc hzd single "path-to-core" import "path-to-json" --format json
```

See `decima-loc --help` for full usage.

## Contributing

Contributions are welcome! Please open an issue or PR.

## License 

This project is licensed under the MIT License - see [LICENSE](LICENSE) for more details.
