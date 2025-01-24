# Eternal Civ13 Verification Server

This is an implementation of a verification server for Valithor's Civ13 Discord bot. It is implemented with minimal external dependencies, only using a JSON file to store data. This is to ensure that the server can be run on any platform with minimal setup.

## Installation

1. Clone the repo
2. Ensure that rust is installed, Rust Nightly is preferred.
3. Run `cargo build --release` to build the server
4. Server binary will be target/release/verify-server. Run the binary to start the server.

## Usage

The server listens on localhost using port 8010 by default. It has the option of using a token to protect writes to it's data. If the token is set to changeme, it is considered unused and the server will only run on localhost, and it is not used. If the token is set to anything else and the server is not localhost, it must be provided.

The server will work with Civilizationbot out of the box.

The server is configured through a `.env` file. The first time the server is run, a `.env` file will be created with default settings.

## License

The server is provided under the AGPL-3.0 license. See LICENSE for more information.