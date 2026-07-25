import {
  CompletionItemProvider,
  CompletionItem,
  CompletionItemKind,
  Position,
  TextDocument,
  CancellationToken,
  CompletionContext,
  ProviderResult,
  CompletionList,
  SnippetString,
  workspace
} from 'vscode';
import {
  FUSION_KEYWORDS,
  FUSION_TYPES,
  FUSION_STDLIB,
  FUSION_BUILTIN_MACROS
} from './language';

export class FusionCompletionProvider implements CompletionItemProvider {
  provideCompletionItems(
    document: TextDocument,
    position: Position,
    token: CancellationToken,
    context: CompletionContext
  ): ProviderResult<CompletionItem[] | CompletionList> {
    const lineText = document.lineAt(position).text;
    const textUntilPosition = lineText.substring(0, position.character);
    const wordRange = document.getWordRangeAtPosition(position);

    if (!wordRange && textUntilPosition.trim().length > 0) {
      return this.getCompletionsForContext(document, position, textUntilPosition);
    }

    const items: CompletionItem[] = [];

    items.push(...this.getKeywordCompletions());
    items.push(...this.getTypeCompletions());
    items.push(...this.getSnippetCompletions());
    items.push(...this.getBuiltinMacroCompletions());

    const enableStdlib = workspace.getConfiguration('fusion').get<boolean>('enableStdlibCompletions', true);
    if (enableStdlib) {
      items.push(...this.getStdlibCompletions());
    }

    return items;
  }

  private getCompletionsForContext(
    document: TextDocument,
    position: Position,
    textUntilPosition: string
  ): CompletionItem[] {
    const items: CompletionItem[] = [];

    if (textUntilPosition.includes('.')) {
      return this.getMemberCompletions(document, position);
    }

    if (textUntilPosition.includes('::')) {
      return this.getPathCompletions(document, position);
    }

    if (textUntilPosition.match(/->\s*$/)) {
      return this.getTypeCompletions();
    }

    if (textUntilPosition.match(/:\s*$/)) {
      return this.getTypeCompletions();
    }

    if (textUntilPosition.match(/#\[.*$/)) {
      items.push(...this.getAttributeCompletions());
      return items;
    }

    return items;
  }

  private getMemberCompletions(document: TextDocument, position: Position): CompletionItem[] {
    const items: CompletionItem[] = [];
    const lineText = document.lineAt(position).text;
    const textBeforeDot = lineText.substring(0, position.character - 1).trim();

    const stdlibMethods = [
      { name: 'len', detail: 'fn(&self) -> usize', description: 'Returns the length' },
      { name: 'is_empty', detail: 'fn(&self) -> bool', description: 'Returns true if empty' },
      { name: 'push', detail: 'fn(&mut self, value: T)', description: 'Appends a value' },
      { name: 'pop', detail: 'fn(&mut self) -> Option<T>', description: 'Removes last element' },
      { name: 'contains', detail: 'fn(&self, value: &T) -> bool', description: 'Checks if contains value' },
      { name: 'iter', detail: 'fn(&self) -> Iter<T>', description: 'Returns an iterator' },
      { name: 'into_iter', detail: 'fn(self) -> IntoIter<T>', description: 'Consumes and returns iterator' },
      { name: 'map', detail: 'fn<F, B>(self, f: F) -> Map<B>', description: 'Transforms elements' },
      { name: 'filter', detail: 'fn<P>(self, predicate: P) -> Filter<T>', description: 'Filters elements' },
      { name: 'fold', detail: 'fn<B, F>(self, init: B, f: F) -> B', description: 'Folds into single value' },
      { name: 'collect', detail: 'fn(self) -> C', description: 'Collects into collection' },
      { name: 'clone', detail: 'fn(&self) -> Self', description: 'Returns a clone' },
      { name: 'to_string', detail: 'fn(&self) -> String', description: 'Converts to String' },
      { name: 'as_str', detail: 'fn(&self) -> &str', description: 'Returns as string slice' },
      { name: 'unwrap', detail: 'fn(self) -> T', description: 'Unwraps Option/Result' },
      { name: 'unwrap_or', detail: 'fn(self, default: T) -> T', description: 'Unwraps with default' },
      { name: 'expect', detail: 'fn(self, msg: &str) -> T', description: 'Unwraps with message' },
      { name: 'is_some', detail: 'fn(&self) -> bool', description: 'Checks if Option is Some' },
      { name: 'is_none', detail: 'fn(&self) -> bool', description: 'Checks if Option is None' },
      { name: 'is_ok', detail: 'fn(&self) -> bool', description: 'Checks if Result is Ok' },
      { name: 'is_err', detail: 'fn(&self) -> bool', description: 'Checks if Result is Err' },
      { name: 'and_then', detail: 'fn<U, F>(self, op: F) -> Result<U>', description: 'Chains Result operations' },
      { name: 'map_err', detail: 'fn<F, O>(self, op: F) -> Result<O>', description: 'Maps error variant' },
      { name: 'ok', detail: 'fn(self) -> Option<T>', description: 'Converts Result to Option' },
      { name: 'err', detail: 'fn(self) -> Option<E>', description: 'Returns error as Option' }
    ];

    for (const method of stdlibMethods) {
      const item = new CompletionItem(method.name, CompletionItemKind.Method);
      item.detail = method.detail;
      item.documentation = method.description;
      item.insertText = new SnippetString(`${method.name}($1)`);
      item.commitCharacters = ['('];
      items.push(item);
    }

    return items;
  }

  private getPathCompletions(document: TextDocument, position: Position): CompletionItem[] {
    const items: CompletionItem[] = [];
    const modules = [
      'math', 'quantum', 'hybrid', 'pqc', 'tensor', 'autodiff',
      'collections', 'io', 'string', 'convert', 'iter', 'option', 'result',
      'supernova', 'cortex', 'haft', 'fiber',
      'kubernetes', 'faas',
      'interop', 'crypto'
    ];

    for (const mod of modules) {
      const item = new CompletionItem(mod, CompletionItemKind.Module);
      item.detail = `module ${mod}`;
      items.push(item);
    }

    return items;
  }

  private getAttributeCompletions(): CompletionItem[] {
    const items: CompletionItem[] = [];
    const attributes = [
      { name: 'test', description: 'Marks a test function' },
      { name: 'derive', description: 'Automatically derives traits' },
      { name: 'cfg', description: 'Conditional compilation' },
      { name: 'allow', description: 'Suppress lint warnings' },
      { name: 'warn', description: 'Set lint warning level' },
      { name: 'deny', description: 'Set lint error level' },
      { name: 'inline', description: 'Control function inlining' },
      { name: 'cold', description: 'Marks function as unlikely to be called' },
      { name: 'must_use', description: 'Warns if result is unused' },
      { name: 'repr', description: 'Controls memory layout' },
      { name: 'quantum', description: 'Quantum-specific attribute' },
      { name: 'hybrid', description: 'Hybrid cryptography attribute' },
      { name: 'ai', description: 'AI/ML-specific attribute' },
      { name: 'cloud', description: 'Cloud deployment attribute' },
      { name: 'interop', description: 'Language interop attribute' }
    ];

    for (const attr of attributes) {
      const item = new CompletionItem(attr.name, CompletionItemKind.Property);
      item.detail = `#${attr.name}`;
      item.documentation = attr.description;
      item.insertText = new SnippetString(`${attr.name}($1)`);
      items.push(item);
    }

    return items;
  }

  private getKeywordCompletions(): CompletionItem[] {
    const items: CompletionItem[] = [];
    const keywordSnippets: Record<string, { snippet: string; detail: string; doc: string }> = {
      'fn': { snippet: 'fn ${1:name}(${2:params}) -> ${3:Type} {\n    $0\n}', detail: 'fn name(params) -> Type { body }', doc: 'Function definition' },
      'let': { snippet: 'let ${1:name}: ${2:Type} = ${3:value};', detail: 'let name: Type = value;', doc: 'Variable binding' },
      'mut': { snippet: 'let mut ${1:name}: ${2:Type} = ${3:value};', detail: 'let mut name: Type = value;', doc: 'Mutable variable binding' },
      'const': { snippet: 'const ${1:NAME}: ${2:Type} = ${3:value};', detail: 'const NAME: Type = value;', doc: 'Constant definition' },
      'struct': { snippet: 'struct ${1:Name} {\n    ${2:field}: ${3:Type},\n}', detail: 'struct Name { field: Type, }', doc: 'Struct definition' },
      'enum': { snippet: 'enum ${1:Name} {\n    ${2:Variant1},\n    ${3:Variant2}(${4:Type}),\n}', detail: 'enum Name { Variant1, Variant2(Type), }', doc: 'Enum definition' },
      'trait': { snippet: 'trait ${1:Name} {\n    fn ${2:method}(&self${3:, params}) -> ${4:Type};\n}', detail: 'trait Name { fn method(&self) -> Type; }', doc: 'Trait definition' },
      'impl': { snippet: 'impl ${1:Type} {\n    $0\n}', detail: 'impl Type { ... }', doc: 'Implementation block' },
      'match': { snippet: 'match ${1:value} {\n    ${2:pattern} => ${3:result},\n    _ => ${4:fallback},\n}', detail: 'match value { pattern => result, _ => fallback, }', doc: 'Pattern matching' },
      'if': { snippet: 'if ${1:condition} {\n    $0\n}', detail: 'if condition { body }', doc: 'Conditional expression' },
      'else': { snippet: 'else {\n    $0\n}', detail: 'else { body }', doc: 'Else branch' },
      'while': { snippet: 'while ${1:condition} {\n    $0\n}', detail: 'while condition { body }', doc: 'While loop' },
      'for': { snippet: 'for ${1:item} in ${2:iter} {\n    $0\n}', detail: 'for item in iter { body }', doc: 'For loop' },
      'loop': { snippet: 'loop {\n    $0\n}', detail: 'loop { body }', doc: 'Infinite loop' },
      'return': { snippet: 'return ${1:value};', detail: 'return value;', doc: 'Return statement' },
      'pub': { snippet: 'pub ${1:item}', detail: 'pub item', doc: 'Public visibility' },
      'use': { snippet: 'use ${1:path};', detail: 'use path;', doc: 'Use declaration' },
      'mod': { snippet: 'mod ${1:name};', detail: 'mod name;', doc: 'Module declaration' },
      'extern': { snippet: 'extern "${1:C}" {\n    $0\n}', detail: 'extern "C" { ... }', doc: 'Foreign function interface' },
      'where': { snippet: 'where ${1:T}: ${2:Trait}', detail: 'where T: Trait', doc: 'Where clause' },
      'async': { snippet: 'async ${1:expr}', detail: 'async expr', doc: 'Async expression' },
      'await': { snippet: '${1:expr}.await', detail: 'expr.await', doc: 'Await expression' },
      // Quantum
      'qubit': { snippet: 'qubit ${1:name} = ${2:quantum_init}();', detail: 'qubit name = quantum_init();', doc: 'Qubit declaration' },
      'quantum': { snippet: 'quantum ${1:fn_name}(${2:params}) {\n    $0\n}', detail: 'quantum fn_name(params) { body }', doc: 'Quantum function' },
      'circuit': { snippet: 'circuit ${1:name} {\n    ${2:gate} ${3:qubit};\n}', detail: 'circuit name { gate qubit; }', doc: 'Quantum circuit definition' },
      'measure': { snippet: 'measure ${1:qubit};', detail: 'measure qubit;', doc: 'Measurement operation' },
      'gate': { snippet: 'gate ${1:name}(${2:qubit}) {\n    $0\n}', detail: 'gate name(qubit) { body }', doc: 'Custom quantum gate' },
      'hadamard': { snippet: 'hadamard ${1:qubit};', detail: 'hadamard qubit;', doc: 'Hadamard gate' },
      'pauli_x': { snippet: 'pauli_x ${1:qubit};', detail: 'pauli_x qubit;', doc: 'Pauli-X gate (NOT)' },
      'pauli_y': { snippet: 'pauli_y ${1:qubit};', detail: 'pauli_y qubit;', doc: 'Pauli-Y gate' },
      'pauli_z': { snippet: 'pauli_z ${1:qubit};', detail: 'pauli_z qubit;', doc: 'Pauli-Z gate (phase flip)' },
      'cnot': { snippet: 'cnot ${1:control}, ${2:target};', detail: 'cnot control, target;', doc: 'Controlled-NOT gate' },
      'swap': { snippet: 'swap ${1:qubit_a}, ${2:qubit_b};', detail: 'swap qubit_a, qubit_b;', doc: 'SWAP gate' },
      'toffoli': { snippet: 'toffoli ${1:ctrl1}, ${2:ctrl2}, ${3:target};', detail: 'toffoli ctrl1, ctrl2, target;', doc: 'Toffoli (CCX) gate' },
      'rx': { snippet: 'rx(${1:theta}) ${2:qubit};', detail: 'rx(theta) qubit;', doc: 'Rotation-X gate' },
      'ry': { snippet: 'ry(${1:theta}) ${2:qubit};', detail: 'ry(theta) qubit;', doc: 'Rotation-Y gate' },
      'rz': { snippet: 'rz(${1:theta}) ${2:qubit};', detail: 'rz(theta) qubit;', doc: 'Rotation-Z gate' },
      'simulate': { snippet: 'simulate ${1:circuit}(${2:shots});', detail: 'simulate circuit(shots);', doc: 'Run quantum simulation' },
      // PQC / Hybrid
      'hybrid': { snippet: 'hybrid ${1:fn_name}(${2:params}) -> ${3:Type} {\n    $0\n}', detail: 'hybrid fn_name(params) -> Type { body }', doc: 'Hybrid crypto function' },
      'kem': { snippet: 'kem ${1:keygen_fn}() -> ${2:KeyPair} {\n    $0\n}', detail: 'kem keygen_fn() -> KeyPair { body }', doc: 'KEM key encapsulation' },
      'pqc': { snippet: 'pqc ${1:fn_name}(${2:params}) {\n    $0\n}', detail: 'pqc fn_name(params) { body }', doc: 'Post-quantum crypto function' },
      'dilithium': { snippet: 'dilithium ${1:keygen_fn}() -> ${2:DilithiumKey} {\n    $0\n}', detail: 'dilithium keygen_fn() -> DilithiumKey', doc: 'Dilithium signature scheme' },
      'kyber': { snippet: 'kyber ${1:keygen_fn}() -> ${2:KyberKey} {\n    $0\n}', detail: 'kyber keygen_fn() -> KyberKey', doc: 'Kyber key encapsulation' },
      'neuralseal': { snippet: 'neuralseal ${1:fn_name}(${2:params}) {\n    $0\n}', detail: 'neuralseal fn_name(params) { body }', doc: 'NeuralSeal encryption' },
      'encrypt': { snippet: 'encrypt(${1:key}, ${2:data});', detail: 'encrypt(key, data);', doc: 'Encrypt data' },
      'decrypt': { snippet: 'decrypt(${1:key}, ${2:ciphertext});', detail: 'decrypt(key, ciphertext);', doc: 'Decrypt ciphertext' },
      'sign': { snippet: 'sign(${1:key}, ${2:message});', detail: 'sign(key, message);', doc: 'Sign a message' },
      'verify': { snippet: 'verify(${1:signature}, ${2:message});', detail: 'verify(signature, message);', doc: 'Verify a signature' },
      // AI/ML
      'tensor': { snippet: 'tensor ${1:name}: ${2:Tensor} = ${3:zeros}([${4:shape}]);', detail: 'tensor name: Tensor = zeros([shape]);', doc: 'Tensor declaration' },
      'autodiff': { snippet: 'autodiff ${1:fn_name}(${2:params}) {\n    $0\n}', detail: 'autodiff fn_name(params) { body }', doc: 'Automatic differentiation function' },
      'train': { snippet: 'train ${1:model}(${2:data}, ${3:epochs});', detail: 'train model(data, epochs);', doc: 'Train a model' },
      'model': { snippet: 'model ${1:name} {\n    ${2:layers}\n}', detail: 'model name { layers }', doc: 'Model definition' },
      'inference': { snippet: 'inference ${1:model}(${2:input});', detail: 'inference model(input);', doc: 'Run inference' },
      'forward': { snippet: 'forward(${1:model}, ${2:input});', detail: 'forward(model, input);', doc: 'Forward pass' },
      'backward': { snippet: 'backward(${1:loss});', detail: 'backward(loss);', doc: 'Backward pass (backpropagation)' },
      'loss': { snippet: 'loss ${1:name} = ${2:mse_loss}(${3:predicted}, ${4:target});', detail: 'loss name = mse_loss(predicted, target);', doc: 'Loss computation' },
      'optimizer': { snippet: 'optimizer ${1:name} = ${2:adam}(${3:model.parameters()});', detail: 'optimizer name = adam(model.parameters());', doc: 'Optimizer definition' },
      // Runtime
      'supernova': { snippet: 'supernova {\n    $0\n}', detail: 'supernova { body }', doc: 'Supernova runtime block' },
      'cortex': { snippet: 'cortex {\n    ${1:intent}\n    $0\n}', detail: 'cortex { intent; body }', doc: 'Cortex AI reasoning engine' },
      'intent': { snippet: 'intent "${1:description}" {\n    $0\n}', detail: 'intent "description" { body }', doc: 'Intent declaration for Cortex' },
      'haft': { snippet: 'haft ${1:scheduler} {\n    $0\n}', detail: 'haft scheduler { body }', doc: 'Haft scheduler block' },
      'fiber': { snippet: 'fiber ${1:name} {\n    $0\n}', detail: 'fiber name { body }', doc: 'Fiber concurrency unit' },
      'schedule': { snippet: 'schedule(${1:task});', detail: 'schedule(task);', doc: 'Schedule a task' },
      'dispatch': { snippet: 'dispatch(${1:task});', detail: 'dispatch(task);', doc: 'Dispatch a task' },
      // Cloud
      'kubernetes': { snippet: 'kubernetes ${1:cluster_name} {\n    $0\n}', detail: 'kubernetes cluster_name { body }', doc: 'Kubernetes cluster config' },
      'deploy': { snippet: 'deploy ${1:service} to ${2:target};', detail: 'deploy service to target;', doc: 'Deploy a service' },
      'faas': { snippet: 'faas ${1:fn_name}(${2:params}) -> ${3:Type} {\n    $0\n}', detail: 'faas fn_name(params) -> Type { body }', doc: 'Function-as-a-Service definition' },
      'container': { snippet: 'container ${1:name} {\n    ${2:image}\n    $0\n}', detail: 'container name { image; ... }', doc: 'Container definition' },
      'pod': { snippet: 'pod ${1:name} {\n    ${2:container}\n    $0\n}', detail: 'pod name { container; ... }', doc: 'Pod definition' },
      'service': { snippet: 'service ${1:name} {\n    ${2:config}\n    $0\n}', detail: 'service name { config; ... }', doc: 'Service definition' },
      // Interop
      'python': { snippet: 'python ${1:script}(${2:args});', detail: 'python script(args);', doc: 'Call Python function' },
      'javascript': { snippet: 'javascript ${1:code}(${2:args});', detail: 'javascript code(args);', doc: 'Evaluate JavaScript' },
      'java': { snippet: 'java ${1:class_name}::${2:method}(${3:args});', detail: 'java class_name::method(args);', doc: 'Call Java method' },
      'py_import': { snippet: 'py_import ${1:module_name};', detail: 'py_import module_name;', doc: 'Import Python module' },
      'js_eval': { snippet: 'js_eval ${1:expression};', detail: 'js_eval expression;', doc: 'Evaluate JavaScript expression' },
      'jvm_call': { snippet: 'jvm_call ${1:class_name}::${2:method}(${3:args});', detail: 'jvm_call class_name::method(args);', doc: 'Call JVM method' }
    };

    for (const keyword of FUSION_KEYWORDS) {
      const item = new CompletionItem(keyword, CompletionItemKind.Keyword);
      item.detail = `keyword ${keyword}`;
      item.documentation = `Fusion keyword: ${keyword}`;

      if (keywordSnippets[keyword]) {
        const snippet = keywordSnippets[keyword];
        item.insertText = new SnippetString(snippet.snippet);
        item.detail = snippet.detail;
        item.documentation = snippet.doc;
      }

      items.push(item);
    }

    return items;
  }

  private getTypeCompletions(): CompletionItem[] {
    const items: CompletionItem[] = [];
    const typeDocs: Record<string, string> = {
      'i8': '8-bit signed integer',
      'i16': '16-bit signed integer',
      'i32': '32-bit signed integer',
      'i64': '64-bit signed integer',
      'i128': '128-bit signed integer',
      'isize': 'Platform-dependent signed integer',
      'u8': '8-bit unsigned integer',
      'u16': '16-bit unsigned integer',
      'u32': '32-bit unsigned integer',
      'u64': '64-bit unsigned integer',
      'u128': '128-bit unsigned integer',
      'usize': 'Platform-dependent unsigned integer',
      'f32': '32-bit floating point',
      'f64': '64-bit floating point',
      'bool': 'Boolean (true/false)',
      'char': 'Unicode scalar value',
      'str': 'String slice',
      'String': 'Growable string',
      'Vec': 'Growable vector',
      'Option': 'Optional value (Some/None)',
      'Result': 'Operation result (Ok/Err)',
      'Box': 'Heap-allocated value',
      'Rc': 'Reference-counted pointer',
      'Arc': 'Atomically reference-counted pointer',
      'HashMap': 'Hash map',
      // Quantum types
      'QuantumState': 'Quantum state representation',
      'QuantumRegister': 'Quantum register',
      'QuantumCircuit': 'Quantum circuit',
      'Qubit': 'Quantum bit',
      'Gate': 'Quantum gate',
      'Backend': 'Quantum simulation backend',
      // PQC types
      'HybridKey': 'Hybrid cryptographic key',
      'KemCiphertext': 'KEM ciphertext',
      'PqcSignature': 'Post-quantum signature',
      'DilithiumKey': 'Dilithium signing key',
      'KyberKey': 'Kyber encapsulation key',
      'NeuralSeal': 'NeuralSeal encryption key',
      // AI/ML types
      'Tensor': 'Multi-dimensional array',
      'AutodiffGraph': 'Automatic differentiation graph',
      'Model': 'AI/ML model',
      'Optimizer': 'Training optimizer',
      'LossFunction': 'Loss function for training',
      'Dataset': 'Training dataset',
      // Runtime types
      'SupernovaRuntime': 'Supernova async runtime',
      'CortexEngine': 'Cortex AI reasoning engine',
      'IntentGraph': 'Intent graph for Cortex',
      'HaftScheduler': 'Haft task scheduler',
      'Fiber': 'Lightweight concurrency fiber',
      // Cloud types
      'KubernetesCluster': 'Kubernetes cluster',
      'Container': 'Docker container',
      'Pod': 'Kubernetes pod',
      'Service': 'Network service',
      'FaasEndpoint': 'Function-as-a-Service endpoint',
      // Interop types
      'PyModule': 'Python module handle',
      'JsValue': 'JavaScript value handle',
      'JvmClass': 'JVM class reference'
    };

    for (const type of FUSION_TYPES) {
      const item = new CompletionItem(type, CompletionItemKind.TypeParameter);
      item.detail = `type ${type}`;
      item.documentation = typeDocs[type] || `Type: ${type}`;
      items.push(item);
    }

    return items;
  }

  private getStdlibCompletions(): CompletionItem[] {
    const items: CompletionItem[] = [];

    const stdlibDocs: Record<string, Record<string, string>> = {
      'quantum': {
        'hadamard': 'Apply Hadamard gate to create superposition',
        'cnot': 'Controlled-NOT gate for entanglement',
        'toffoli': 'Toffoli (CCX) gate - doubly-controlled NOT',
        'phase': 'Apply phase gate',
        'rotate_x': 'Rotation around X-axis',
        'rotate_y': 'Rotation around Y-axis',
        'rotate_z': 'Rotation around Z-axis',
        'measure': 'Measure qubit, collapsing to classical',
        'create_qubit': 'Initialize a new qubit',
        'entangle': 'Entangle two qubits',
        'superpose': 'Put qubit in superposition',
        'pauli_x': 'Pauli-X gate (quantum NOT)',
        'pauli_y': 'Pauli-Y gate',
        'pauli_z': 'Pauli-Z gate (phase flip)',
        'swap': 'Swap two qubit states',
        'rx': 'Rotation-X gate with angle theta',
        'ry': 'Rotation-Y gate with angle theta',
        'rz': 'Rotation-Z gate with angle theta',
        'simulate': 'Run circuit simulation with N shots',
        'create_circuit': 'Create a new quantum circuit',
        'run_circuit': 'Execute a quantum circuit'
      },
      'pqc': {
        'dilithium_keygen': 'Generate Dilithium signing keypair',
        'dilithium_sign': 'Sign message with Dilithium',
        'dilithium_verify': 'Verify Dilithium signature',
        'kyber_keygen': 'Generate Kyber encapsulation keypair',
        'kyber_encapsulate': 'Encapsulate shared secret with Kyber',
        'kyber_decapsulate': 'Decapsulate shared secret with Kyber',
        'neuralseal_encrypt': 'Encrypt with NeuralSeal scheme',
        'neuralseal_decrypt': 'Decrypt NeuralSeal ciphertext'
      },
      'tensor': {
        'zeros': 'Create tensor filled with zeros',
        'ones': 'Create tensor filled with ones',
        'full': 'Create tensor filled with constant',
        'arange': 'Create tensor from range',
        'linspace': 'Create tensor with linearly spaced values',
        'rand': 'Create tensor with random uniform values',
        'randn': 'Create tensor with random normal values',
        'matmul': 'Matrix multiplication',
        'transpose': 'Transpose tensor dimensions',
        'reshape': 'Reshape tensor',
        'softmax': 'Apply softmax activation',
        'relu': 'Apply ReLU activation',
        'sigmoid': 'Apply sigmoid activation',
        'tanh': 'Apply tanh activation',
        'mse_loss': 'Mean squared error loss',
        'cross_entropy': 'Cross-entropy loss',
        'gradient': 'Compute gradient',
        'backward': 'Backward pass through autodiff graph',
        'sgd_step': 'SGD optimizer step',
        'adam_step': 'Adam optimizer step'
      },
      'autodiff': {
        'grad': 'Compute gradient of function',
        'jacobian': 'Compute Jacobian matrix',
        'hessian': 'Compute Hessian matrix',
        'vjp': 'Vector-Jacobian product',
        'jvp': 'Jacobian-Vector product',
        'tape': 'Record operations on autodiff tape',
        'enable_grad': 'Enable gradient computation',
        'disable_grad': 'Disable gradient computation'
      },
      'supernova': {
        'spawn': 'Spawn a new async task',
        'join': 'Join (await) a task handle',
        'yield_now': 'Yield current task',
        'sleep': 'Sleep for duration',
        'block_on': 'Block on future synchronously',
        'spawn_blocking': 'Spawn blocking task on threadpool',
        'task_local': 'Access task-local storage',
        'JoinHandle': 'Handle to a spawned task'
      },
      'cortex': {
        'intent': 'Declare AI intent',
        'plan': 'Create execution plan',
        'execute': 'Execute planned actions',
        'observe': 'Observe environment state',
        'adapt': 'Adapt strategy based on feedback',
        'reason': 'Perform logical reasoning',
        'infer': 'Perform inference',
        'cluster': 'Cluster related intents',
        'topology': 'Get intent graph topology',
        'shard': 'Shard intent graph across nodes'
      },
      'haft': {
        'schedule': 'Schedule task on Haft scheduler',
        'dispatch': 'Dispatch task to worker',
        'prioritize': 'Set task priority',
        'preempt': 'Preempt running task',
        'yield': 'Yield execution slot',
        'queue_depth': 'Get current queue depth',
        'worker_count': 'Get active worker count',
        'set_affinity': 'Set CPU affinity for worker'
      },
      'fiber': {
        'new': 'Create a new fiber',
        'spawn_fiber': 'Spawn a fiber for execution',
        'block_on_fiber': 'Block current fiber on future',
        'fiber_id': 'Get current fiber ID',
        'fiber_local': 'Access fiber-local storage',
        'yield_fiber': 'Yield current fiber'
      },
      'kubernetes': {
        'deploy_pod': 'Deploy a pod to cluster',
        'create_service': 'Create a K8s service',
        'list_pods': 'List all pods in namespace',
        'scale_deployment': 'Scale a deployment',
        'delete_pod': 'Delete a pod',
        'get_logs': 'Get pod logs',
        'apply_yaml': 'Apply YAML manifest'
      },
      'faas': {
        'invoke': 'Invoke a FaaS function',
        'deploy_function': 'Deploy a function',
        'list_functions': 'List deployed functions',
        'delete_function': 'Delete a function',
        'get_invocation_logs': 'Get function invocation logs',
        'set_memory_limit': 'Set function memory limit',
        'set_timeout': 'Set function timeout'
      },
      'interop': {
        'py_import': 'Import a Python module',
        'js_eval': 'Evaluate JavaScript code',
        'jvm_call': 'Call a JVM method',
        'py_call': 'Call a Python function',
        'js_call': 'Call a JavaScript function',
        'jvm_new': 'Instantiate a JVM object',
        'to_python': 'Convert value to Python object',
        'to_javascript': 'Convert value to JS value',
        'to_java': 'Convert value to Java object',
        'from_python': 'Convert Python object to Fusion',
        'from_javascript': 'Convert JS value to Fusion',
        'from_java': 'Convert Java object to Fusion'
      },
      'crypto': {
        'gen_keypair': 'Generate key pair',
        'sign_message': 'Sign a message',
        'verify_signature': 'Verify signature',
        'encrypt_data': 'Encrypt data',
        'decrypt_data': 'Decrypt data',
        'derive_shared_secret': 'Derive shared secret',
        'hash_sha256': 'SHA-256 hash',
        'hash_sha512': 'SHA-512 hash',
        'hmac_sign': 'HMAC sign',
        'hmac_verify': 'HMAC verify',
        'aes_encrypt': 'AES encrypt',
        'aes_decrypt': 'AES decrypt'
      }
    };

    for (const [module, functions] of FUSION_STDLIB) {
      for (const fn of functions) {
        const item = new CompletionItem(`${module}::${fn}`, CompletionItemKind.Function);
        item.detail = `fn ${module}::${fn}()`;
        item.documentation = stdlibDocs[module]?.[fn] || `Function in ${module} module`;
        item.insertText = new SnippetString(`${fn}($1)`);
        item.commitCharacters = ['('];
        items.push(item);
      }
    }

    return items;
  }

  private getSnippetCompletions(): CompletionItem[] {
    const items: CompletionItem[] = [];

    const snippets: Array<{
      name: string;
      label: string;
      detail: string;
      documentation: string;
      snippet: string;
    }> = [
      {
        name: 'fn',
        label: 'fn',
        detail: 'Function definition',
        documentation: 'Creates a new function',
        snippet: 'fn ${1:name}(${2:params}) -> ${3:Type} {\n    $0\n}'
      },
      {
        name: 'struct',
        label: 'struct',
        detail: 'Struct definition',
        documentation: 'Creates a new struct',
        snippet: 'struct ${1:Name} {\n    ${2:field}: ${3:Type},\n}'
      },
      {
        name: 'enum',
        label: 'enum',
        detail: 'Enum definition',
        documentation: 'Creates a new enum',
        snippet: 'enum ${1:Name} {\n    ${2:Variant1},\n    ${3:Variant2}(${4:Type}),\n}'
      },
      {
        name: 'impl',
        label: 'impl',
        detail: 'Implementation block',
        documentation: 'Creates an implementation block',
        snippet: 'impl ${1:Type} {\n    $0\n}'
      },
      {
        name: 'trait',
        label: 'trait',
        detail: 'Trait definition',
        documentation: 'Defines a new trait',
        snippet: 'trait ${1:Name} {\n    fn ${2:method}(&self${3:, params}) -> ${4:Type};\n}'
      },
      {
        name: 'match',
        label: 'match',
        detail: 'Match expression',
        documentation: 'Pattern matching expression',
        snippet: 'match ${1:value} {\n    ${2:pattern} => ${3:result},\n    _ => ${4:fallback},\n}'
      },
      {
        name: 'test',
        label: 'test',
        detail: 'Test function',
        documentation: 'Creates a test function',
        snippet: '#[test]\nfn ${1:test_name}() {\n    $0\n}'
      },
      {
        name: 'test_module',
        label: 'test module',
        detail: 'Test module',
        documentation: 'Creates a test module',
        snippet: '#[cfg(test)]\nmod tests {\n    use super::*;\n\n    #[test]\n    fn ${1:test_name}() {\n        $0\n    }\n}'
      }
    ];

    for (const snippet of snippets) {
      const item = new CompletionItem(snippet.label, CompletionItemKind.Snippet);
      item.detail = snippet.detail;
      item.documentation = snippet.documentation;
      item.insertText = new SnippetString(snippet.snippet);
      item.kind = CompletionItemKind.Snippet;
      items.push(item);
    }

    return items;
  }

  private getBuiltinMacroCompletions(): CompletionItem[] {
    const items: CompletionItem[] = [];
    const macroDocs: Record<string, string> = {
      'println!': 'Prints to stdout with newline',
      'print!': 'Prints to stdout',
      'eprintln!': 'Prints to stderr with newline',
      'eprint!': 'Prints to stderr',
      'format!': 'Returns formatted String',
      'vec!': 'Creates a new Vec',
      'panic!': 'Panics with message',
      'assert!': 'Asserts condition is true',
      'assert_eq!': 'Asserts two values are equal',
      'assert_ne!': 'Asserts two values are not equal',
      'todo!': 'Marks unimplemented code',
      'unimplemented!': 'Marks unimplemented code',
      'unreachable!': 'Marks unreachable code',
      'include!': 'Includes file at compile time',
      'include_str!': 'Includes file as string',
      'include_bytes!': 'Includes file as bytes',
      'env!': 'Reads environment variable',
      'concat!': 'Concatenates literals',
      'stringify!': 'Converts token to string'
    };

    for (const macro of FUSION_BUILTIN_MACROS) {
      const item = new CompletionItem(macro, CompletionItemKind.Function);
      item.detail = `macro ${macro}`;
      item.documentation = macroDocs[macro] || `Built-in macro: ${macro}`;
      items.push(item);
    }

    return items;
  }
}
