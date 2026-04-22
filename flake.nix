{
  description = "ESP32-C6 Rust development environment";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-25.11";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };

        # RISC-V target for ESP32-C6
        target = "riscv32imac-unknown-none-elf";

        # Rust toolchain with necessary components
        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rust-src" "rustfmt" "clippy" ];
          targets = [ target ];
        };

      in
      {
        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            rustToolchain

            # ESP32 specific tools
            espflash        # For flashing
            ldproxy         # Linker proxy for Rust binaries
            esptool         # Espressif flash tool
            esp-generate

            # Additional helpful tools
            gdb             # For debugging
            openocd         # For JTAG debugging
            probe-rs-tools  # Alternative flashing/debugging
            cargo-generate  # For project templates

            rust-analyzer
          ];

          # Environment variables
          RUST_SRC_PATH = "${rustToolchain}/lib/rustlib/src/rust/library";

          # Set target for cargo commands
          shellHook = ''
            echo "🔧 ESP32-C6 Rust Development Environment"
            echo "📦 Target: ${target}"
            echo ""
            echo "Commands:"
            echo "  cargo build --release"
            echo "  cargo run"
            echo "  espflash flash --monitor target/${target}/release/your-project"
            echo ""

            # Export target for convenience
            export ESP_TARGET="${target}"
          '';
        };
      });
}
