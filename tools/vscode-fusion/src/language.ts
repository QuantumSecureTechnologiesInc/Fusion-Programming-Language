import {
  LanguageConfiguration,
  CommentRule,
  AutoClosingPair,
  CharacterPair,
  IndentationRule
} from 'vscode';

export const FUSION_LANGUAGE_CONFIG: LanguageConfiguration = {
  comments: {
    lineComment: '//',
    blockComment: ['/*', '*/']
  },

  brackets: [
    ['{', '}'],
    ['[', ']'],
    ['(', ')']
  ],

  autoClosingPairs: [
    { open: '{', close: '}' },
    { open: '[', close: ']' },
    { open: '(', close: ')' },
    { open: '"', close: '"' },
    { open: "'", close: "'" },
    { open: '`', close: '`' },
    { open: '/*', close: '*/' }
  ],

  surroundingPairs: [
    { open: '{', close: '}' },
    { open: '[', close: ']' },
    { open: '(', close: ')' },
    { open: '"', close: '"' },
    { open: "'", close: "'" }
  ],

  folding: {
    markers: {
      start: /^\s*\/\/.*#region\b/,
      end: /^\s*\/\/.*#endregion\b/
    }
  },

  indentationRules: {
    increaseIndentPattern: /^\s*(fn|struct|enum|trait|impl|match|if|else|while|for|loop|block|unsafe|extern|mod|quantum|circuit|hybrid|pqc|kubernetes|faas|supernova|cortex|haft|fiber|model)\b.*[{\(]\s*$/,
    decreaseIndentPattern: /^\s*[}\)]/
  },

  wordPattern: /(-?\d*\.\d\w*)|([^\`\~\!\@\#\%\^\&\*\(\)\-\=\+\[\{\]\}\\\|\;\:\'\"\,\.\<\>\/\?\s]+)/g
};

export const FUSION_KEYWORDS: string[] = [
  'fn', 'let', 'mut', 'const', 'static',
  'struct', 'enum', 'trait', 'impl', 'type',
  'match', 'if', 'else', 'while', 'for', 'loop', 'break', 'continue', 'return',
  'pub', 'use', 'mod', 'extern',
  'self', 'Self', 'super', 'crate',
  'true', 'false',
  'as', 'in', 'where', 'async', 'await',
  'unsafe', 'ref', 'move',
  // Quantum
  'qubit', 'quantum', 'circuit', 'measure', 'gate', 'hadamard', 'pauli_x', 'pauli_y', 'pauli_z',
  'cnot', 'swap', 'toffoli', 'rx', 'ry', 'rz', 'simulate',
  // PQC / Hybrid
  'hybrid', 'kem', 'dilithium', 'kyber', 'pqc', 'neuralseal', 'encrypt', 'decrypt', 'sign', 'verify',
  // AI/ML
  'tensor', 'autodiff', 'train', 'model', 'inference', 'forward', 'backward', 'loss', 'optimizer',
  // Runtime
  'supernova', 'cortex', 'intent', 'haft', 'fiber', 'schedule', 'dispatch',
  // Cloud
  'kubernetes', 'deploy', 'faas', 'container', 'pod', 'service',
  // Interop
  'python', 'javascript', 'java', 'py_import', 'js_eval', 'jvm_call'
];

export const FUSION_TYPES: string[] = [
  'i8', 'i16', 'i32', 'i64', 'i128', 'isize',
  'u8', 'u16', 'u32', 'u64', 'u128', 'usize',
  'f32', 'f64',
  'bool', 'char', 'str', 'String',
  'Vec', 'Option', 'Result', 'Box', 'Rc', 'Arc',
  'HashMap', 'HashSet', 'BTreeMap',
  // Quantum types
  'QuantumState', 'QuantumRegister', 'QuantumCircuit', 'Qubit', 'Gate', 'Backend',
  // PQC types
  'HybridKey', 'KemCiphertext', 'PqcSignature', 'DilithiumKey', 'KyberKey', 'NeuralSeal',
  // AI/ML types
  'Tensor', 'AutodiffGraph', 'Model', 'Optimizer', 'LossFunction', 'Dataset',
  // Runtime types
  'SupernovaRuntime', 'CortexEngine', 'IntentGraph', 'HaftScheduler', 'Fiber',
  // Cloud types
  'KubernetesCluster', 'Container', 'Pod', 'Service', 'FaasEndpoint',
  // Interop types
  'PyModule', 'JsValue', 'JvmClass'
];

export const FUSION_STDLIB: Map<string, string[]> = new Map([
  ['math', ['abs', 'sqrt', 'pow', 'min', 'max', 'ceil', 'floor', 'round']],
  ['quantum', ['hadamard', 'cnot', 'toffoli', 'phase', 'rotate_x', 'rotate_y', 'rotate_z', 'measure', 'create_qubit', 'entangle', 'superpose', 'pauli_x', 'pauli_y', 'pauli_z', 'swap', 'rx', 'ry', 'rz', 'simulate', 'create_circuit', 'run_circuit']],
  ['hybrid', ['encrypt', 'decrypt', 'sign', 'verify', 'keygen', 'encapsulate', 'decapsulate']],
  ['pqc', ['dilithium_keygen', 'dilithium_sign', 'dilithium_verify', 'kyber_keygen', 'kyber_encapsulate', 'kyber_decapsulate', 'neuralseal_encrypt', 'neuralseal_decrypt']],
  ['tensor', ['zeros', 'ones', 'full', 'arange', 'linspace', 'rand', 'randn', 'matmul', 'transpose', 'reshape', 'softmax', 'relu', 'sigmoid', 'tanh', 'mse_loss', 'cross_entropy', 'gradient', 'backward', 'sgd_step', 'adam_step']],
  ['autodiff', ['grad', 'jacobian', 'hessian', 'vjp', 'jvp', 'tape', 'enable_grad', 'disable_grad']],
  ['supernova', ['spawn', 'join', 'yield_now', 'sleep', 'block_on', 'spawn_blocking', 'task_local', 'JoinHandle']],
  ['cortex', ['intent', 'plan', 'execute', 'observe', 'adapt', 'reason', 'infer', 'cluster', 'topology', 'shard']],
  ['haft', ['schedule', 'dispatch', 'prioritize', 'preempt', 'yield', 'queue_depth', 'worker_count', 'set_affinity']],
  ['fiber', ['new', 'spawn_fiber', 'block_on_fiber', 'fiber_id', 'fiber_local', 'yield_fiber']],
  ['kubernetes', ['deploy_pod', 'create_service', 'list_pods', 'scale_deployment', 'delete_pod', 'get_logs', 'apply_yaml']],
  ['faas', ['invoke', 'deploy_function', 'list_functions', 'delete_function', 'get_invocation_logs', 'set_memory_limit', 'set_timeout']],
  ['collections', ['vec', 'hashmap', 'btreemap', 'btree_set', 'linked_list', 'vec_deque']],
  ['io', ['print', 'println', 'eprint', 'eprintln', 'format', 'read_line', 'stdin', 'stdout', 'stderr']],
  ['string', ['len', 'push', 'pop', 'contains', 'starts_with', 'ends_with', 'split', 'trim', 'to_upper', 'to_lower', 'replace', 'parse']],
  ['convert', ['into', 'from', 'try_into', 'try_from', 'as_ref', 'to_string', 'as_str']],
  ['iter', ['map', 'filter', 'fold', 'reduce', 'collect', 'enumerate', 'zip', 'chain', 'take', 'skip', 'any', 'all', 'find', 'for_each', 'sum', 'count', 'max', 'min']],
  ['option', ['is_some', 'is_none', 'unwrap', 'unwrap_or', 'unwrap_or_else', 'map', 'and_then', 'ok_or', 'ok_or_else']],
  ['result', ['is_ok', 'is_err', 'unwrap', 'unwrap_or', 'unwrap_or_else', 'map', 'map_err', 'and_then', 'ok', 'err', 'expect']],
  ['interop', ['py_import', 'js_eval', 'jvm_call', 'py_call', 'js_call', 'jvm_new', 'to_python', 'to_javascript', 'to_java', 'from_python', 'from_javascript', 'from_java']],
  ['crypto', ['gen_keypair', 'sign_message', 'verify_signature', 'encrypt_data', 'decrypt_data', 'derive_shared_secret', 'hash_sha256', 'hash_sha512', 'hmac_sign', 'hmac_verify', 'aes_encrypt', 'aes_decrypt']]
]);

export const FUSION_BUILTIN_MACROS: string[] = [
  'println!',
  'print!',
  'eprintln!',
  'eprint!',
  'format!',
  'vec!',
  'panic!',
  'assert!',
  'assert_eq!',
  'assert_ne!',
  'debug_assert!',
  'debug_assert_eq!',
  'debug_assert_ne!',
  'todo!',
  'unimplemented!',
  'unreachable!',
  'include!',
  'include_str!',
  'include_bytes!',
  'env!',
  'option_env!',
  'concat!',
  'module_path!',
  'file!',
  'line!',
  'column!',
  'stringify!'
];
