#!/usr/bin/env python3
"""Fix all broken stdlib .fu files in one pass."""
import os

BASE = os.path.dirname(os.path.abspath(__file__))
Q = '"'

def write_fu(name, content):
    path = os.path.join(BASE, name)
    with open(path, 'w', encoding='utf-8') as f:
        f.write(content)
    print(f"  wrote {name}: {len(content.splitlines())} lines")

# === json.fu ===
write_fu('stdlib/json.fu', f"""// stdlib/json.fu - JSON utilities for Fusion

extern fn fusion_json_escape(s: string) -> string;
extern fn fusion_json_kv_string(k: string, v: string) -> string;
extern fn fusion_json_kv_int(k: string, v: int) -> string;
extern fn fusion_fmt_int(v: int) -> string;
extern fn fusion_strlen(s: string) -> int;

// JsonValue - simple tagged JSON value (16 bytes)
// kind: 0=Null, 1=BoolFalse, 2=BoolTrue, 3=Int, 4=String
const JSON_NULL: int = 0;
const JSON_BOOL_FALSE: int = 1;
const JSON_BOOL_TRUE: int = 2;
const JSON_INT: int = 3;
const JSON_STRING: int = 4;

struct JsonValue {{
    kind: int,
    int_val: int,
    string_val: string,
}}

impl JsonValue {{
    fn null() -> JsonValue {{
        return JsonValue {{ kind: 0, int_val: 0, string_val: {Q}{Q} }};
    }}
    fn from_bool(v: bool) -> JsonValue {{
        if v {{ return JsonValue {{ kind: 2, int_val: 1, string_val: {Q}{Q} }}; }}
        return JsonValue {{ kind: 1, int_val: 0, string_val: {Q}{Q} }};
    }}
    fn from_int(v: int) -> JsonValue {{
        return JsonValue {{ kind: 3, int_val: v, string_val: {Q}{Q} }};
    }}
    fn from_string(v: string) -> JsonValue {{
        return JsonValue {{ kind: 4, int_val: 0, string_val: v }};
    }}
    fn is_null(jv: JsonValue) -> bool {{
        return jv.kind == 0;
    }}
    fn as_bool(jv: JsonValue) -> bool {{
        return jv.kind == 2;
    }}
    fn as_int(jv: JsonValue) -> int {{
        return jv.int_val;
    }}
    fn as_string(jv: JsonValue) -> string {{
        return jv.string_val;
    }}
}}

// Serialization
fn json_serialize(jv: JsonValue) -> string {{
    if jv.kind == 0 {{ return {Q}null{Q}; }}
    if jv.kind == 1 {{ return {Q}false{Q}; }}
    if jv.kind == 2 {{ return {Q}true{Q}; }}
    if jv.kind == 3 {{ return fusion_fmt_int(jv.int_val); }}
    if jv.kind == 4 {{
        return {Q}\\{Q}{Q} + fusion_json_escape(jv.string_val) + {Q}\\{Q}{Q};
    }}
    return {Q}null{Q};
}}

fn json_kv_str(key: string, value: string) -> string {{
    return fusion_json_kv_string(key, value);
}}

fn json_kv_int(key: string, value: int) -> string {{
    return fusion_json_kv_int(key, value);
}}

fn json_kv_bool(key: string, value: bool) -> string {{
    let vstr: string = {Q}false{Q};
    if value {{ vstr = {Q}true{Q}; }}
    return {Q}\\{Q}{Q} + fusion_json_escape(key) + {Q}\\{Q}:{Q} + vstr;
}}

fn json_object(content: string) -> string {{
    return {Q}{{{Q} + content + {Q}}}{Q};
}}

fn json_array(content: string) -> string {{
    return {Q}[{Q} + content + {Q}]{Q};
}}

fn json_join(a: string, b: string) -> string {{
    if fusion_strlen(a) == 0 {{ return b; }}
    if fusion_strlen(b) == 0 {{ return a; }}
    return a + {Q},{Q} + b;
}}
""")

# === result.fu ===
# The LLVM "Entry block" error might be from struct with string field.
# Keep struct simple: just is_ok (int) + err_value (string) = 12 bytes
write_fu('stdlib/result.fu', f"""// stdlib/result.fu - Result types for Fusion
// Concrete Result types for error handling.
// ResultBoolString: 16 bytes (is_ok: int, ok_value: int, err_value: string)

struct ResultBoolString {{
    is_ok: int,
    ok_value: int,
    err_value: string,
}}

impl ResultBoolString {{
    fn ok(val: int) -> ResultBoolString {{
        return ResultBoolString {{ is_ok: 1, ok_value: val, err_value: {Q}{Q} }};
    }}
    fn err(error: string) -> ResultBoolString {{
        return ResultBoolString {{ is_ok: 0, ok_value: 0, err_value: error }};
    }}
    fn is_ok(r: ResultBoolString) -> bool {{
        return r.is_ok != 0;
    }}
    fn is_err(r: ResultBoolString) -> bool {{
        return r.is_ok == 0;
    }}
    fn unwrap(r: ResultBoolString) -> int {{
        return r.ok_value;
    }}
    fn unwrap_err(r: ResultBoolString) -> string {{
        return r.err_value;
    }}
    fn unwrap_or(r: ResultBoolString, default: int) -> int {{
        if r.is_ok != 0 {{
            return r.ok_value;
        }} else {{
            return default;
        }}
    }}
}}

fn result_bool_ok(val: int) -> ResultBoolString {{
    return ResultBoolString {{ is_ok: 1, ok_value: val, err_value: {Q}{Q} }};
}}
fn result_bool_err(error: string) -> ResultBoolString {{
    return ResultBoolString {{ is_ok: 0, ok_value: 0, err_value: error }};
}}

// NOTE: Generic Result<T, E> requires generics support.
""")

# === option.fu ===
# OptionString has string field - may cause LLVM issue
# Keep OptionInt and OptionBool (no strings), make OptionString handle-based
write_fu('stdlib/option.fu', f"""// stdlib/option.fu - Option types for Fusion
// Concrete Option types since the compiler lacks generics.

// ============================================================
// OptionInt
// ============================================================
struct OptionInt {{
    has_value: int,
    value: int,
}}

impl OptionInt {{
    fn some(val: int) -> OptionInt {{
        return OptionInt {{ has_value: 1, value: val }};
    }}
    fn none() -> OptionInt {{
        return OptionInt {{ has_value: 0, value: 0 }};
    }}
    fn is_some(opt: OptionInt) -> bool {{
        return opt.has_value != 0;
    }}
    fn is_none(opt: OptionInt) -> bool {{
        return opt.has_value == 0;
    }}
    fn unwrap(opt: OptionInt) -> int {{
        return opt.value;
    }}
    fn unwrap_or(opt: OptionInt, default: int) -> int {{
        if opt.has_value != 0 {{
            return opt.value;
        }} else {{
            return default;
        }}
    }}
}}

fn option_int_some(val: int) -> OptionInt {{
    return OptionInt {{ has_value: 1, value: val }};
}}
fn option_int_none() -> OptionInt {{
    return OptionInt {{ has_value: 0, value: 0 }};
}}
fn option_int_unwrap(opt: OptionInt) -> int {{
    return opt.value;
}}

// ============================================================
// OptionBool
// ============================================================
struct OptionBool {{
    has_value: int,
    value: int,
}}

impl OptionBool {{
    fn some(val: bool) -> OptionBool {{
        if val {{ return OptionBool {{ has_value: 1, value: 1 }}; }}
        return OptionBool {{ has_value: 1, value: 0 }};
    }}
    fn none() -> OptionBool {{
        return OptionBool {{ has_value: 0, value: 0 }};
    }}
    fn is_some(opt: OptionBool) -> bool {{
        return opt.has_value != 0;
    }}
    fn is_none(opt: OptionBool) -> bool {{
        return opt.has_value == 0;
    }}
    fn unwrap(opt: OptionBool) -> bool {{
        return opt.value != 0;
    }}
}}

fn option_bool_some(val: bool) -> OptionBool {{
    if val {{ return OptionBool {{ has_value: 1, value: 1 }}; }}
    return OptionBool {{ has_value: 1, value: 0 }};
}}
fn option_bool_none() -> OptionBool {{
    return OptionBool {{ has_value: 0, value: 0 }};
}}
fn option_bool_unwrap(opt: OptionBool) -> bool {{
    return opt.value != 0;
}}

// ============================================================
// OptionString (handle-based to avoid string-in-struct issues)
// ============================================================
extern fn fusion_opt_str_create(s: string) -> int;
extern fn fusion_opt_str_get(h: int) -> string;
extern fn fusion_opt_str_has(h: int) -> int;

struct OptionString {{
    handle: int,
    _pad: int,
}}

impl OptionString {{
    fn some(val: string) -> OptionString {{
        let h: int = fusion_opt_str_create(val);
        return OptionString {{ handle: h, _pad: 0 }};
    }}
    fn none() -> OptionString {{
        return OptionString {{ handle: -1, _pad: 0 }};
    }}
    fn is_some(opt: OptionString) -> bool {{
        return opt.handle >= 0;
    }}
    fn is_none(opt: OptionString) -> bool {{
        return opt.handle < 0;
    }}
    fn unwrap(opt: OptionString) -> string {{
        return fusion_opt_str_get(opt.handle);
    }}
}}

fn option_string_some(val: string) -> OptionString {{
    let h: int = fusion_opt_str_create(val);
    return OptionString {{ handle: h, _pad: 0 }};
}}
fn option_string_none() -> OptionString {{
    return OptionString {{ handle: -1, _pad: 0 }};
}}
fn option_string_unwrap(opt: OptionString) -> string {{
    return fusion_opt_str_get(opt.handle);
}}

// NOTE: Generic Option<T> requires generics support.
""")

print("All files written successfully.")
