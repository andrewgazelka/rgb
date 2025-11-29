#!/usr/bin/env python3
import json
import sys
import re

def to_snake(s):
    s = re.sub(r'([A-Z]+)([A-Z][a-z])', r'\1_\2', s)
    s = re.sub(r'([a-z\d])([A-Z])', r'\1_\2', s)
    return s.lower()

def to_pascal(s):
    parts = s.replace('minecraft:', '').replace('/', '_').split('_')
    return ''.join(x.title() for x in parts)

KNOWN_TYPES = {
    'i8', 'i16', 'i32', 'i64',
    'f32', 'f64', 'bool',
    'String', 'Uuid', 'Position',
    'Nbt', 'BlockState',
}

def rust_type(t):
    mapping = {
        'i8': 'i8', 'i16': 'i16', 'i32': 'i32', 'i64': 'i64',
        'f32': 'f32', 'f64': 'f64', 'bool': 'bool',
        'String': 'String', 'Uuid': 'Uuid', 'Position': 'Position',
        'Nbt': 'Nbt', 'BlockState': 'BlockState',
    }
    if t in mapping:
        return mapping[t]
    if t.startswith('Vec<') and 'Unknown' not in t:
        return t
    if t.startswith('Option<') and 'Unknown' not in t:
        return t
    # Unknown types as raw bytes with comment (Cow for zero-copy)
    return f"/* {t} */ Cow<'a, [u8]>"

def is_known_type(t):
    """Check if a type is fully known (can be derived)"""
    if t in KNOWN_TYPES:
        return True
    if t.startswith('Vec<'):
        inner = t[4:-1]
        return is_known_type(inner)
    if t.startswith('Option<'):
        inner = t[7:-1]
        return is_known_type(inner)
    return False

def gen_struct(name, fields, packet_id):
    field_types = [rust_type(f['rustType']) for f in fields]
    needs_lifetime = any("Cow<'a" in t for t in field_types)
    all_known = all(is_known_type(f['rustType']) for f in fields)

    lines = [f'/// Packet ID: {packet_id}']
    if all_known:
        lines.append('#[derive(Debug, Clone, Encode, Decode, Serialize, Deserialize)]')
    else:
        lines.append('#[derive(Debug, Clone, Serialize, Deserialize)]')
    if needs_lifetime:
        lines.append(f"pub struct {name}<'a> {{")
    else:
        lines.append(f'pub struct {name} {{')
    for f in fields:
        fname = to_snake(f['name'])
        if fname == 'type':
            fname = 'r#type'
        ftype = rust_type(f['rustType'])
        lines.append(f'    pub {fname}: {ftype},')
    lines.append('}')
    lines.append('')
    return '\n'.join(lines)

def gen_packet_impl(struct_name, packet_id, state, direction, needs_lifetime=False):
    """Generate impl Packet for the struct"""
    state_enum = {
        'handshake': 'Handshaking',
        'status': 'Status',
        'login': 'Login',
        'configuration': 'Configuration',
        'play': 'Play',
    }[state]

    dir_enum = 'Clientbound' if direction == 'clientbound' else 'Serverbound'

    lifetime = "<'_>" if needs_lifetime else ""
    lines = [
        f'impl Packet for {struct_name}{lifetime} {{',
        f'    const ID: i32 = {packet_id};',
        f'    const STATE: State = State::{state_enum};',
        f'    const DIRECTION: Direction = Direction::{dir_enum};',
        '}',
        '',
    ]
    return '\n'.join(lines)

def main():
    ids_file, fields_file, out_dir = sys.argv[1], sys.argv[2], sys.argv[3]

    # Protocol version can be passed as 4th arg, name as 5th
    protocol_version = int(sys.argv[4]) if len(sys.argv) > 4 else None
    protocol_name = sys.argv[5] if len(sys.argv) > 5 else None

    with open(ids_file) as f:
        ids_data = json.load(f)
    with open(fields_file) as f:
        fields_data = json.load(f)

    # Build lookup for fields by class name
    fields_by_class = {}
    for state, dirs in fields_data.items():
        for direction, packets in dirs.items():
            for p in packets:
                fields_by_class[p['className']] = p['fields']

    states = ['handshake', 'status', 'login', 'configuration', 'play']

    for state in states:
        content = [
            f'// Auto-generated from Minecraft - {state}',
            '// Do not edit manually',
            '',
            '#![allow(dead_code)]',
            '',
            'use std::borrow::Cow;',
            'use mc_protocol::{Encode, Decode, Packet, State, Direction, VarInt, Uuid, Position, Nbt, BlockState};',
            'use serde::{Serialize, Deserialize};',
            '',
        ]

        state_ids = ids_data.get(state, {})

        for direction in ['clientbound', 'serverbound']:
            dir_ids = state_ids.get(direction, {})
            if not dir_ids:
                continue

            content.append(f'pub mod {direction} {{')
            content.append('    use super::*;')
            content.append('')

            for pkt_name, pkt_info in sorted(dir_ids.items(), key=lambda x: x[1]['protocol_id']):
                pkt_id = pkt_info['protocol_id']
                clean_name = pkt_name.replace('minecraft:', '').replace('/', '_')
                struct_name = to_pascal(clean_name)

                # Try to find fields - class names are like "ClientboundAddEntityPacket"
                dir_prefix = 'Clientbound' if direction == 'clientbound' else 'Serverbound'
                class_patterns = [
                    f'{dir_prefix}{struct_name}Packet',
                    f'{struct_name}Packet',
                    struct_name,
                ]
                fields = []
                for pattern in class_patterns:
                    if pattern in fields_by_class:
                        fields = fields_by_class[pattern]
                        break

                # Check if struct needs lifetime
                needs_lifetime = False
                if fields:
                    field_types = [rust_type(f['rustType']) for f in fields]
                    needs_lifetime = any("Cow<'a" in t for t in field_types)

                # Generate struct
                if fields:
                    struct_lines = gen_struct(struct_name, fields, pkt_id)
                    for line in struct_lines.split('\n'):
                        content.append(f'    {line}' if line else '')
                else:
                    content.append(f'    /// Packet ID: {pkt_id}')
                    content.append(f'    #[derive(Debug, Clone, Default, Encode, Decode, Serialize, Deserialize)]')
                    content.append(f'    pub struct {struct_name};')
                    content.append('')

                # Generate impl Packet
                impl_lines = gen_packet_impl(struct_name, pkt_id, state, direction, needs_lifetime)
                for line in impl_lines.split('\n'):
                    content.append(f'    {line}' if line else '')

            content.append('}')
            content.append('')

        with open(f'{out_dir}/{state}.rs', 'w') as f:
            f.write('\n'.join(content))

    # Generate lib.rs
    with open(f'{out_dir}/lib.rs', 'w') as f:
        f.write('// Auto-generated Minecraft packet definitions\n')
        f.write('// Run `nix run .#mc-gen` to regenerate\n\n')

        if protocol_version is not None:
            f.write(f'/// Protocol version for this build\n')
            f.write(f'pub const PROTOCOL_VERSION: i32 = {protocol_version};\n\n')

        if protocol_name is not None:
            f.write(f'/// Minecraft version name for this build\n')
            f.write(f'pub const PROTOCOL_NAME: &str = "{protocol_name}";\n\n')

        # Re-export State and Direction for convenience
        f.write('// Re-export protocol types\n')
        f.write('pub use mc_protocol::{State, Direction, Packet};\n\n')

        for s in states:
            f.write(f'pub mod {s};\n')

if __name__ == '__main__':
    main()
