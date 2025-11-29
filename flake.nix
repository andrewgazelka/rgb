{
  description = "RGB - Minecraft server in Rust";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    crane.url = "github:ipetkov/crane";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, flake-utils, crane, rust-overlay }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ rust-overlay.overlays.default ];
        };

        # Use toolchain from rust-toolchain.toml
        rustToolchain = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
        craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;

        # Include JSON data files needed by build.rs
        src = pkgs.lib.cleanSourceWith {
          src = ./.;
          filter = path: type:
            (craneLib.filterCargoSources path type) ||
            (builtins.match ".*\.json$" path != null);
        };

        # Build the Rust mc-server
        mcServer = craneLib.buildPackage {
          inherit src;
          cargoExtraArgs = "-p mc-server";
          strictDeps = true;
        };

        mcVersion = "1.21.11-pre3";

        # Download and cache MC server jar
        downloadMcJar = pkgs.writeShellScriptBin "download-mc-jar" ''
          set -euo pipefail
          VERSION="''${1:-${mcVersion}}"
          CACHE_DIR="''${XDG_CACHE_HOME:-$HOME/.cache}/mc-data-gen"
          mkdir -p "$CACHE_DIR"

          MANIFEST=$(${pkgs.curl}/bin/curl -s "https://piston-meta.mojang.com/mc/game/version_manifest_v2.json")
          VERSION_URL=$(echo "$MANIFEST" | ${pkgs.jq}/bin/jq -r ".versions[] | select(.id == \"$VERSION\") | .url")

          if [ -z "$VERSION_URL" ] || [ "$VERSION_URL" = "null" ]; then
            echo "Error: Version $VERSION not found" >&2
            exit 1
          fi

          VERSION_INFO=$(${pkgs.curl}/bin/curl -s "$VERSION_URL")
          SERVER_URL=$(echo "$VERSION_INFO" | ${pkgs.jq}/bin/jq -r '.downloads.server.url')
          SERVER_JAR="$CACHE_DIR/server-$VERSION.jar"

          if [ ! -f "$SERVER_JAR" ]; then
            ${pkgs.curl}/bin/curl -sL "$SERVER_URL" -o "$SERVER_JAR" >&2
          fi
          echo "$SERVER_JAR"
        '';

        # Run Mojang's data generator
        mcDataGen = pkgs.writeShellScriptBin "mc-data-gen" ''
          set -euo pipefail
          VERSION="''${1:-${mcVersion}}"
          OUTPUT_DIR="''${2:-.}"

          SERVER_JAR=$(${downloadMcJar}/bin/download-mc-jar "$VERSION")
          mkdir -p "$OUTPUT_DIR"
          cd "$OUTPUT_DIR"
          ${pkgs.jdk21}/bin/java -DbundlerMainClass="net.minecraft.data.Main" -jar "$SERVER_JAR" --all
        '';

        # Download unobfuscated client jar (only available for recent snapshots)
        downloadUnobfuscatedClient = pkgs.writeShellScriptBin "download-unobfuscated-client" ''
          set -euo pipefail
          VERSION="''${1:-${mcVersion}}"
          CACHE_DIR="''${XDG_CACHE_HOME:-$HOME/.cache}/mc-data-gen"
          mkdir -p "$CACHE_DIR"

          # Unobfuscated client URLs for known versions
          case "$VERSION" in
            25w45a)
              URL="https://piston-data.mojang.com/v1/objects/26551033b7b935436f3407b85d14cac835e65640/client.jar"
              ;;
            25w46a)
              URL="https://piston-data.mojang.com/v1/objects/2a1e9a8e09a6312adb91bf2a1b1188c57861ab65/client.jar"
              ;;
            1.21.11-pre3)
              URL="https://piston-data.mojang.com/v1/objects/70ab73433d42e46d3838462e724cdb3c0116b0c4/client.jar"
              ;;
            *)
              echo "No unobfuscated client available for $VERSION" >&2
              exit 1
              ;;
          esac

          CLIENT_JAR="$CACHE_DIR/client-unobfuscated-$VERSION.jar"
          if [ ! -f "$CLIENT_JAR" ]; then
            echo "Downloading unobfuscated client for $VERSION..." >&2
            ${pkgs.curl}/bin/curl -sL "$URL" -o "$CLIENT_JAR"
          fi
          echo "$CLIENT_JAR"
        '';

        # Extract packet fields via reflection from unobfuscated client
        packetExtractor = pkgs.writeShellScriptBin "extract-packets" ''
          set -euo pipefail
          VERSION="''${1:-${mcVersion}}"
          OUTPUT="''${2:-packets-extracted.json}"

          CLIENT_JAR=$(${downloadUnobfuscatedClient}/bin/download-unobfuscated-client "$VERSION")

          TEMP_DIR=$(mktemp -d)
          trap "rm -rf $TEMP_DIR" EXIT
          cd "$TEMP_DIR"

          echo "Using unobfuscated client: $CLIENT_JAR" >&2

          # Write the extractor source
          cat > PacketExtractor.java << 'JAVA_EOF'
import java.io.*;
import java.lang.reflect.*;
import java.net.*;
import java.util.*;
import java.util.jar.*;
import java.util.stream.*;

public class PacketExtractor {
    public static void main(String[] args) throws Exception {
        if (args.length < 2) { System.err.println("Usage: PacketExtractor <jar> <output>"); System.exit(1); }
        String jarPath = args[0], outputPath = args[1];
        URL[] urls = { new File(jarPath).toURI().toURL() };
        URLClassLoader loader = new URLClassLoader(urls, PacketExtractor.class.getClassLoader());
        Map<String, Map<String, List<Map<String, Object>>>> result = new LinkedHashMap<>();

        try (JarFile jar = new JarFile(jarPath)) {
            jar.stream().map(e -> e.getName()).filter(n -> n.endsWith(".class") && n.contains("network/protocol"))
               .map(n -> n.replace("/", ".").replace(".class", "")).forEach(className -> {
                try {
                    Class<?> clazz = loader.loadClass(className);
                    if (!isPacket(clazz)) return;
                    Map<String, Object> info = extractInfo(clazz);
                    if (info == null) return;
                    String state = (String) info.get("state"), dir = (String) info.get("direction");
                    result.computeIfAbsent(state, k -> new LinkedHashMap<>())
                          .computeIfAbsent(dir, k -> new ArrayList<>()).add(info);
                } catch (Throwable e) {}
            });
        }
        try (PrintWriter out = new PrintWriter(outputPath)) { out.println(toJson(result, 0)); }
        System.out.println("Extracted to: " + outputPath);
    }

    static boolean isPacket(Class<?> c) {
        for (Class<?> i : c.getInterfaces()) if (i.getSimpleName().equals("Packet")) return true;
        Class<?> s = c.getSuperclass();
        while (s != null && s != Object.class) {
            for (Class<?> i : s.getInterfaces()) if (i.getSimpleName().equals("Packet")) return true;
            s = s.getSuperclass();
        }
        return false;
    }

    static Map<String, Object> extractInfo(Class<?> c) {
        Map<String, Object> info = new LinkedHashMap<>();
        info.put("className", c.getSimpleName());
        String pkg = c.getPackage().getName();
        info.put("state", pkg.contains("handshaking") ? "handshake" : pkg.contains("status") ? "status" :
                         pkg.contains("login") ? "login" : pkg.contains("configuration") ? "configuration" :
                         pkg.contains("game") || pkg.contains("play") ? "play" : "common");
        info.put("direction", (pkg.contains("clientbound") || c.getSimpleName().startsWith("Clientbound")) ? "clientbound" :
                              (pkg.contains("serverbound") || c.getSimpleName().startsWith("Serverbound")) ? "serverbound" : "unknown");

        List<Map<String, String>> fields = new ArrayList<>();
        if (c.isRecord()) {
            for (RecordComponent rc : c.getRecordComponents()) {
                Map<String, String> f = new LinkedHashMap<>();
                f.put("name", rc.getName());
                f.put("javaType", rc.getType().getSimpleName());
                f.put("rustType", toRust(rc.getType(), rc.getGenericType()));
                fields.add(f);
            }
        } else {
            for (Field field : c.getDeclaredFields()) {
                if (Modifier.isStatic(field.getModifiers()) || Modifier.isTransient(field.getModifiers())) continue;
                Map<String, String> f = new LinkedHashMap<>();
                f.put("name", field.getName());
                f.put("javaType", field.getType().getSimpleName());
                f.put("rustType", toRust(field.getType(), field.getGenericType()));
                fields.add(f);
            }
        }
        info.put("fields", fields);
        return info;
    }

    static String toRust(Class<?> t, Type g) {
        if (t == boolean.class || t == Boolean.class) return "bool";
        if (t == byte.class || t == Byte.class) return "i8";
        if (t == short.class || t == Short.class) return "i16";
        if (t == int.class || t == Integer.class) return "i32";
        if (t == long.class || t == Long.class) return "i64";
        if (t == float.class || t == Float.class) return "f32";
        if (t == double.class || t == Double.class) return "f64";
        if (t == String.class) return "String";
        if (t.getSimpleName().equals("UUID")) return "Uuid";
        if (t.getSimpleName().equals("BlockPos")) return "Position";
        if (t.getSimpleName().equals("Component")) return "String";
        if (t.getSimpleName().equals("ResourceLocation")) return "String";
        if (t.getSimpleName().contains("Tag")) return "Nbt";
        if (t.isArray()) return t.getComponentType() == byte.class ? "Vec<u8>" : "Vec<" + toRust(t.getComponentType(), t.getComponentType()) + ">";
        if (List.class.isAssignableFrom(t)) return "Vec<Unknown>";
        if (Optional.class.isAssignableFrom(t)) return "Option<Unknown>";
        if (t.isEnum()) return t.getSimpleName();
        return t.getSimpleName();
    }

    static String toJson(Object o, int ind) {
        String p = "  ".repeat(ind), p1 = "  ".repeat(ind + 1);
        if (o == null) return "null";
        if (o instanceof String) return "\"" + ((String)o).replace("\\", "\\\\").replace("\"", "\\\"") + "\"";
        if (o instanceof Number || o instanceof Boolean) return o.toString();
        if (o instanceof Map) {
            Map<?,?> m = (Map<?,?>)o;
            if (m.isEmpty()) return "{}";
            StringBuilder sb = new StringBuilder("{\n");
            boolean first = true;
            for (Map.Entry<?,?> e : m.entrySet()) {
                if (!first) sb.append(",\n");
                first = false;
                sb.append(p1).append("\"").append(e.getKey()).append("\": ").append(toJson(e.getValue(), ind + 1));
            }
            return sb.append("\n").append(p).append("}").toString();
        }
        if (o instanceof List) {
            List<?> l = (List<?>)o;
            if (l.isEmpty()) return "[]";
            StringBuilder sb = new StringBuilder("[\n");
            boolean first = true;
            for (Object e : l) { if (!first) sb.append(",\n"); first = false; sb.append(p1).append(toJson(e, ind + 1)); }
            return sb.append("\n").append(p).append("]").toString();
        }
        return "\"" + o.toString() + "\"";
    }
}
JAVA_EOF

          ${pkgs.jdk21}/bin/javac PacketExtractor.java
          ${pkgs.jdk21}/bin/java -cp ".:$CLIENT_JAR" PacketExtractor "$CLIENT_JAR" "$OUTPUT"
        '';

        # Run real MC server in offline mode for packet capture
        runMcServer = pkgs.writeShellScriptBin "run-mc-server" ''
          set -euo pipefail
          VERSION="''${1:-${mcVersion}}"
          PORT="''${2:-25566}"

          SERVER_JAR=$(${downloadMcJar}/bin/download-mc-jar "$VERSION")

          TEMP_DIR=$(mktemp -d)
          trap "rm -rf $TEMP_DIR" EXIT
          cd "$TEMP_DIR"

          # Create eula.txt
          echo "eula=true" > eula.txt

          # Create server.properties for offline mode
          cat > server.properties << EOF
          server-port=$PORT
          online-mode=false
          level-type=minecraft:flat
          spawn-protection=0
          max-players=1
          view-distance=4
          simulation-distance=4
          EOF

          echo "Starting Minecraft server on port $PORT (offline mode, superflat)..."
          echo "Server JAR: $SERVER_JAR"
          exec ${pkgs.jdk21}/bin/java -Xmx1G -jar "$SERVER_JAR" nogui
        '';

        # Packet capture proxy - sits between client and real server
        packetProxy = pkgs.writeShellScriptBin "packet-proxy" ''
          set -euo pipefail
          LISTEN_PORT="''${1:-25565}"
          TARGET_PORT="''${2:-25566}"
          OUTPUT_FILE="''${3:-packets.bin}"

          echo "Packet proxy: listening on $LISTEN_PORT, forwarding to localhost:$TARGET_PORT"
          echo "Saving raw packets to: $OUTPUT_FILE"

          # Use socat with tee to capture traffic
          ${pkgs.socat}/bin/socat -v TCP-LISTEN:$LISTEN_PORT,fork,reuseaddr \
            SYSTEM:"tee -a $OUTPUT_FILE.client | ${pkgs.socat}/bin/socat - TCP:localhost:$TARGET_PORT | tee -a $OUTPUT_FILE.server"
        '';

        # Extract and update JSON data files for packet generation
        mcGen = pkgs.writeShellScriptBin "mc-gen" ''
          set -euo pipefail
          VERSION="''${1:-${mcVersion}}"
          DATA_DIR="./crates/mc-packets/data"

          TEMP_DIR=$(mktemp -d)
          trap "rm -rf $TEMP_DIR" EXIT

          echo "=== MC Protocol Data Extractor ==="
          echo "Version: $VERSION"
          echo ""

          mkdir -p "$DATA_DIR"

          # Extract packet fields from unobfuscated client
          echo "Extracting packet fields..."
          ${packetExtractor}/bin/extract-packets "$VERSION" "$DATA_DIR/packets-fields.json" 2>/dev/null || echo "{}" > "$DATA_DIR/packets-fields.json"

          # Run Mojang data gen for packet IDs
          echo "Running Mojang data generator for IDs..."
          ${mcDataGen}/bin/mc-data-gen "$VERSION" "$TEMP_DIR" >/dev/null 2>&1
          cp "$TEMP_DIR/generated/reports/packets.json" "$DATA_DIR/packets-ids.json"

          # Extract protocol version from client jar
          CLIENT_JAR=$(${downloadUnobfuscatedClient}/bin/download-unobfuscated-client "$VERSION" 2>/dev/null)
          PROTOCOL_VERSION=$(${pkgs.unzip}/bin/unzip -p "$CLIENT_JAR" version.json 2>/dev/null | ${pkgs.jq}/bin/jq -r '.protocol_version')
          echo "Protocol version: $PROTOCOL_VERSION"

          # Write protocol.json
          cat > "$DATA_DIR/protocol.json" << EOF
          {"version": "$VERSION", "protocol_version": $PROTOCOL_VERSION}
          EOF

          echo ""
          echo "Updated data files in $DATA_DIR"
          echo "Run \`cargo build -p mc-packets\` to rebuild with new data."
        '';

      in {
        packages = {
          default = mcServer;
          mc-server = mcServer;
          mc-data-gen = mcDataGen;
          mc-gen = mcGen;
          extract-packets = packetExtractor;
          download-mc-jar = downloadMcJar;
          run-vanilla-server = runMcServer;
          packet-proxy = packetProxy;
        };

        apps = {
          default = flake-utils.lib.mkApp { drv = mcServer; name = "mc-server"; };
          mc-server = flake-utils.lib.mkApp { drv = mcServer; name = "mc-server"; };
          mc-data-gen = flake-utils.lib.mkApp { drv = mcDataGen; };
          mc-gen = flake-utils.lib.mkApp { drv = mcGen; };
          extract-packets = flake-utils.lib.mkApp { drv = packetExtractor; };
          run-vanilla-server = flake-utils.lib.mkApp { drv = runMcServer; };
          packet-proxy = flake-utils.lib.mkApp { drv = packetProxy; };
        };

        devShells.default = pkgs.mkShell {
          buildInputs = [
            rustToolchain
            pkgs.jdk21
            pkgs.curl
            pkgs.jq
            mcDataGen
            mcGen
            packetExtractor
          ];
        };
      }
    );
}
