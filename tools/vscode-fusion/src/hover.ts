import {
  HoverProvider,
  Hover,
  MarkdownString,
  Position,
  TextDocument,
  CancellationToken,
  ProviderResult,
  workspace
} from 'vscode';
import { FUSION_KEYWORDS, FUSION_TYPES, FUSION_STDLIB } from './language';

export class FusionHoverProvider implements HoverProvider {
  provideHover(
    document: TextDocument,
    position: Position,
    token: CancellationToken
  ): ProviderResult<Hover> {
    const wordRange = document.getWordRangeAtPosition(position);
    if (!wordRange) {
      return null;
    }

    const word = document.getText(wordRange);
    const lineText = document.lineAt(position).text;
    const textBeforeWord = lineText.substring(0, position.character);

    if (textBeforeWord.endsWith('.')) {
      return this.getMemberHover(word, document, position);
    }

    if (textBeforeWord.endsWith('::')) {
      return this.getPathHover(word);
    }

    const keywordHover = this.getKeywordHover(word);
    if (keywordHover) {
      return keywordHover;
    }

    const typeHover = this.getTypeHover(word);
    if (typeHover) {
      return typeHover;
    }

    const macroHover = this.getMacroHover(word);
    if (macroHover) {
      return macroHover;
    }

    const symbolHover = this.getSymbolHover(word, document, position);
    if (symbolHover) {
      return symbolHover;
    }

    return null;
  }

  private getKeywordHover(word: string): Hover | null {
    const keywordDocs: Record<string, string> = {
      // Core keywords
      'fn': '**fn** - Function definition\n\n```fusion\nfn name(params) -> ReturnType {\n    body\n}\n```\n\nFunctions are first-class values. Supports generics, lifetimes, and async.',
      'let': '**let** - Variable binding\n\n```fusion\nlet name: Type = value;\nlet name = value; // type inference\n```\n\nBinds a value to a variable. Variables are immutable by default.',
      'mut': '**mut** - Mutable binding\n\n```fusion\nlet mut name: Type = value;\nname = new_value; // allowed\n```\n\nMakes a variable binding mutable.',
      'const': '**const** - Constant definition\n\n```fusion\nconst NAME: Type = value;\n```\n\nCompile-time constant. Must have explicit type.',
      'struct': '**struct** - Structure definition\n\n```fusion\nstruct Point {\n    x: f64,\n    y: f64,\n}\n```\n\nNamed-field structure type.',
      'enum': '**enum** - Enumeration type\n\n```fusion\nenum Shape {\n    Circle(f64),\n    Rectangle(f64, f64),\n    Point,\n}\n```\n\nAlgebraic data type with variants.',
      'trait': '**trait** - Trait definition\n\n```fusion\ntrait Drawable {\n    fn draw(&self);\n}\n```\n\nDefines shared interface/behavior.',
      'impl': '**impl** - Implementation block\n\n```fusion\nimpl Drawable for Circle {\n    fn draw(&self) {\n        // ...\n    }\n}\n```\n\nImplements methods for a type.',
      'match': '**match** - Pattern matching\n\n```fusion\nmatch value {\n    Pattern1 => result1,\n    Pattern2(x) => result2,\n    _ => default,\n}\n```\n\nExhaustive pattern matching.',
      'if': '**if** - Conditional expression\n\n```fusion\nif condition {\n    body\n} else {\n    else_body\n}\n```\n\nReturns a value in Fusion.',
      'else': '**else** - Else branch\n\nUsed with `if` for alternative execution path.',
      'while': '**while** - While loop\n\n```fusion\nwhile condition {\n    body\n}\n```\n\nLoops while condition is true.',
      'for': '**for** - For loop\n\n```fusion\nfor item in collection {\n    body\n}\n```\n\nIterates over a collection or range.',
      'loop': '**loop** - Infinite loop\n\n```fusion\nloop {\n    body\n    if done { break; }\n}\n```\n\nLoops indefinitely until `break`.',
      'return': '**return** - Return value\n\n```fusion\nreturn value;\n```\n\nReturns from the current function.',
      'pub': '**pub** - Public visibility\n\n```fusion\npub fn public_fn() {}\npub struct PublicStruct {}\n```\n\nMakes items visible outside their module.',
      'use': '**use** - Use declaration\n\n```fusion\nuse std::collections::HashMap;\nuse crate::module::{Item1, Item2};\n```\n\nImports items into scope.',
      'mod': '**mod** - Module declaration\n\n```fusion\nmod my_module;\nmod inner {\n    // ...\n}\n```\n\nDeclares or defines a module.',
      'extern': '**extern** - Foreign function interface\n\n```fusion\nextern "C" {\n    fn foreign_fn();\n}\n```\n\nDeclares external functions.',
      'async': '**async** - Async expression\n\n```fusion\nasync fn fetch_data() -> Data {\n    // async body\n}\n```\n\nDefines asynchronous function.',
      'await': '**await** - Await expression\n\n```fusion\nlet result = async_operation().await;\n```\n\nWaits for async operation to complete.',

      // Quantum keywords
      'qubit': '**qubit** - Quantum bit\n\n```fusion\nqubit q = |0>;\n```\n\nDeclares a quantum bit in superposition state.',
      'quantum': '**quantum** - Quantum function\n\n```fusion\nquantum apply_h(q: Qubit) {\n    hadamard(q);\n}\n```\n\nDeclares a quantum computing function.',
      'circuit': '**circuit** - Quantum circuit\n\n```fusion\ncircuit bell_state {\n    hadamard q0;\n    cnot q0, q1;\n}\n```\n\nDefines a quantum circuit composition.',
      'measure': '**measure** - Quantum measurement\n\n```fusion\nmeasure qubit;\nlet classical_bit = measure q;\n```\n\nCollapses quantum state to classical value.',
      'gate': '**gate** - Custom quantum gate\n\n```fusion\ngate my_gate(q: Qubit) {\n    // gate operations\n}\n```\n\nDefines a custom quantum gate.',
      'hadamard': '**hadamard** - Hadamard gate\n\n```fusion\nhadamard qubit;\n```\n\nPuts qubit into equal superposition: H|0⟩ = (|0⟩+|1⟩)/√2',
      'pauli_x': '**pauli_x** - Pauli-X gate (NOT)\n\n```fusion\npauli_x qubit;\n```\n\nFlips qubit state: X|0⟩ = |1⟩, X|1⟩ = |0⟩',
      'pauli_y': '**pauli_y** - Pauli-Y gate\n\n```fusion\npauli_y qubit;\n```\n\nRotates qubit around Y-axis of Bloch sphere.',
      'pauli_z': '**pauli_z** - Pauli-Z gate (phase flip)\n\n```fusion\npauli_z qubit;\n```\n\nApplies phase flip: Z|1⟩ = -|1⟩',
      'cnot': '**cnot** - Controlled-NOT gate\n\n```fusion\ncnot control, target;\n```\n\nFlips target if control is |1⟩. Essential for entanglement.',
      'swap': '**swap** - SWAP gate\n\n```fusion\nswap qubit_a, qubit_b;\n```\n\nSwaps states of two qubits.',
      'toffoli': '**toffoli** - Toffoli (CCX) gate\n\n```fusion\ntoffoli ctrl1, ctrl2, target;\n```\n\nDoubly-controlled NOT. Flips target only when both controls are |1⟩.',
      'rx': '**rx** - Rotation-X gate\n\n```fusion\nrx(theta) qubit;\n```\n\nRotates qubit around X-axis by angle theta.',
      'ry': '**ry** - Rotation-Y gate\n\n```fusion\nry(theta) qubit;\n```\n\nRotates qubit around Y-axis by angle theta.',
      'rz': '**rz** - Rotation-Z gate\n\n```fusion\nrz(theta) qubit;\n```\n\nRotates qubit around Z-axis by angle theta.',
      'simulate': '**simulate** - Quantum simulation\n\n```fusion\nsimulate my_circuit(1024);\n```\n\nRuns the circuit on a simulator with specified number of shots.',

      // PQC / Hybrid keywords
      'hybrid': '**hybrid** - Hybrid cryptography\n\n```fusion\nhybrid encrypt(key: HybridKey, data: &[u8]) -> Vec<u8> {\n    // ...\n}\n```\n\nCombines classical and post-quantum cryptography.',
      'kem': '**kem** - Key Encapsulation Mechanism\n\n```fusion\nkem keygen() -> (PublicKey, SecretKey) {\n    // ...\n}\n```\n\nKey encapsulation for secure key exchange.',
      'pqc': '**pqc** - Post-Quantum Cryptography\n\n```fusion\npqc sign(key: SecretKey, msg: &[u8]) -> Signature {\n    // ...\n}\n```\n\nPost-quantum cryptographic operations.',
      'dilithium': '**dilithium** - Dilithium signature scheme\n\n```fusion\ndilithium keygen() -> DilithiumKey { ... }\ndilithium_sign(key, msg) -> PqcSignature { ... }\n```\n\nLattice-based digital signature (NIST PQC standard).',
      'kyber': '**kyber** - Kyber key encapsulation\n\n```fusion\nkyber keygen() -> KyberKey { ... }\nkyber_encapsulate(pubkey) -> KemCiphertext { ... }\n```\n\nLattice-based KEM (NIST PQC standard).',
      'neuralseal': '**neuralseal** - NeuralSeal encryption\n\n```fusion\nneuralseal encrypt(key, data) -> Vec<u8> { ... }\n```\n\nNeural network-based homomorphic encryption.',
      'encrypt': '**encrypt** - Encrypt data\n\n```fusion\nencrypt(key, plaintext) -> ciphertext;\n```\n\nEncrypts data with given key.',
      'decrypt': '**decrypt** - Decrypt data\n\n```fusion\ndecrypt(key, ciphertext) -> plaintext;\n```\n\nDecrypts ciphertext with given key.',
      'sign': '**sign** - Sign a message\n\n```fusion\nsign(key, message) -> signature;\n```\n\nCreates a digital signature for a message.',
      'verify': '**verify** - Verify a signature\n\n```fusion\nverify(signature, message) -> bool;\n```\n\nVerifies a digital signature against a message.',

      // AI/ML keywords
      'tensor': '**tensor** - Multi-dimensional array\n\n```fusion\ntensor x: Tensor = zeros([2, 3]);\ntensor y: Tensor = ones([4, 4]);\n```\n\nDeclares a tensor for numerical computation.',
      'autodiff': '**autodiff** - Automatic differentiation\n\n```fusion\nautodiff loss_fn(x: Tensor) -> Tensor {\n    // gradient tracking enabled\n}\n```\n\nEnables automatic gradient computation.',
      'train': '**train** - Model training\n\n```fusion\ntrain my_model(dataset, epochs=100);\n```\n\nTrains a machine learning model on data.',
      'model': '**model** - Neural network model\n\n```fusion\nmodel classifier {\n    linear(input, 128) -> relu -> linear(10) -> softmax\n}\n```\n\nDefines a neural network architecture.',
      'inference': '**inference** - Run inference\n\n```fusion\nlet result = inference(model, input_tensor);\n```\n\nRuns forward pass to get model prediction.',
      'forward': '**forward** - Forward pass\n\n```fusion\nlet output = forward(model, input);\n```\n\nExecutes forward pass through the model.',
      'backward': '**backward** - Backward pass\n\n```fusion\nbackward(loss);\n```\n\nComputes gradients via backpropagation.',
      'loss': '**loss** - Loss function\n\n```fusion\nloss total = cross_entropy(predictions, labels);\n```\n\nComputes training loss.',
      'optimizer': '**optimizer** - Optimizer\n\n```fusion\noptimizer opt = adam(model.parameters(), lr=0.001);\n```\n\nDefines optimization strategy for training.',

      // Runtime keywords
      'supernova': '**supernova** - Async runtime block\n\n```fusion\nsupernova {\n    let result = spawn(async_task).await;\n}\n```\n\nEntry point for Supernova async runtime.',
      'cortex': '**cortex** - AI reasoning engine\n\n```fusion\ncortex {\n    intent "analyze data" {\n        plan();\n        execute();\n    }\n}\n```\n\nAI-powered reasoning and planning engine.',
      'intent': '**intent** - AI intent declaration\n\n```fusion\nintent "optimize performance" {\n    // reasoning steps\n}\n```\n\nDeclares an AI intent for Cortex to reason about.',
      'haft': '**haft** - Haft scheduler\n\n```fusion\nhaft scheduler {\n    schedule(task_a, priority=high);\n    schedule(task_b, priority=low);\n}\n```\n\nHigh-performance task scheduler.',
      'fiber': '**fiber** - Lightweight concurrency unit\n\n```fusion\nfiber compute {\n    // lightweight concurrent computation\n}\n```\n\nGreen thread / fiber for cooperative concurrency.',
      'schedule': '**schedule** - Schedule a task\n\n```fusion\nschedule(my_task);\n```\n\nAdds task to the Haft scheduler queue.',
      'dispatch': '**dispatch** - Dispatch a task\n\n```fusion\ndispatch(my_task);\n```\n\nImmediately dispatches task to a worker thread.',

      // Cloud keywords
      'kubernetes': '**kubernetes** - K8s cluster config\n\n```fusion\nkubernetes my_cluster {\n    pod webserver {\n        container nginx\n    }\n}\n```\n\nDefines Kubernetes infrastructure.',
      'deploy': '**deploy** - Deploy service\n\n```fusion\ndeploy web_service to kubernetes;\n```\n\nDeploys a service to a target platform.',
      'faas': '**faas** - Function-as-a-Service\n\n```fusion\nfaas handler(req: Request) -> Response {\n    // serverless function\n}\n```\n\nDefines a serverless function.',
      'container': '**container** - Container definition\n\n```fusion\ncontainer app {\n    image \"nginx:latest\"\n    port 80\n}\n```\n\nDefines a container configuration.',
      'pod': '**pod** - Kubernetes pod\n\n```fusion\npod web {\n    container app\n    container sidecar\n}\n```\n\nDefines a pod containing containers.',
      'service': '**service** - Network service\n\n```fusion\nservice api {\n    port 8080\n    target pod/web\n}\n```\n\nDefines a network service.',

      // Interop keywords
      'python': '**python** - Call Python\n\n```fusion\npython my_script(data);\n```\n\nInvokes a Python function.',
      'javascript': '**javascript** - Evaluate JavaScript\n\n```fusion\njavascript process(data);\n```\n\nEvaluates JavaScript code.',
      'java': '**java** - Call Java method\n\n```fusion\njava MyClass::myMethod(args);\n```\n\nInvokes a Java/JVM method.',
      'py_import': '**py_import** - Import Python module\n\n```fusion\npy_import numpy;\n```\n\nImports a Python module into Fusion scope.',
      'js_eval': '**js_eval** - Evaluate JS expression\n\n```fusion\njs_eval JSON.stringify(obj);\n```\n\nEvaluates a JavaScript expression.',
      'jvm_call': '**jvm_call** - Call JVM method\n\n```fusion\njvm_call MyClass::staticMethod(args);\n```\n\nCalls a method on the JVM.'
    };

    const doc = keywordDocs[word];
    if (!doc) {
      return null;
    }

    const markdown = new MarkdownString(doc);
    return new Hover(markdown);
  }

  private getTypeHover(word: string): Hover | null {
    const typeDocs: Record<string, string> = {
      'i8': '**i8** - 8-bit signed integer\n\nRange: -128 to 127\n\n```fusion\nlet x: i8 = 42;\n```',
      'i16': '**i16** - 16-bit signed integer\n\nRange: -32,768 to 32,767\n\n```fusion\nlet x: i16 = 1000;\n```',
      'i32': '**i32** - 32-bit signed integer\n\nRange: -2^31 to 2^31-1\n\n```fusion\nlet x: i32 = 100000;\n```',
      'i64': '**i64** - 64-bit signed integer\n\nRange: -2^63 to 2^63-1\n\n```fusion\nlet x: i64 = 1000000000;\n```',
      'i128': '**i128** - 128-bit signed integer\n\nRange: -2^127 to 2^127-1\n\n```fusion\nlet x: i128 = 1000000000000000000;\n```',
      'isize': '**isize** - Platform-dependent signed integer\n\n32-bit on 32-bit systems, 64-bit on 64-bit systems.',
      'u8': '**u8** - 8-bit unsigned integer\n\nRange: 0 to 255\n\n```fusion\nlet x: u8 = 255;\n```',
      'u16': '**u16** - 16-bit unsigned integer\n\nRange: 0 to 65,535\n\n```fusion\nlet x: u16 = 60000;\n```',
      'u32': '**u32** - 32-bit unsigned integer\n\nRange: 0 to 2^32-1\n\n```fusion\nlet x: u32 = 4000000000;\n```',
      'u64': '**u64** - 64-bit unsigned integer\n\nRange: 0 to 2^64-1\n\n```fusion\nlet x: u64 = 18000000000;\n```',
      'u128': '**u128** - 128-bit unsigned integer\n\nRange: 0 to 2^128-1',
      'usize': '**usize** - Platform-dependent unsigned integer\n\n32-bit on 32-bit systems, 64-bit on 64-bit systems. Used for indexing.',
      'f32': '**f32** - 32-bit floating point\n\nIEEE 754 single-precision.\n\n```fusion\nlet x: f32 = 3.14;\n```',
      'f64': '**f64** - 64-bit floating point\n\nIEEE 754 double-precision.\n\n```fusion\nlet x: f64 = 3.14159265358979;\n```',
      'bool': '**bool** - Boolean type\n\nValues: `true` or `false`\n\n```fusion\nlet active: bool = true;\n```',
      'char': '**char** - Character type\n\nUnicode scalar value (4 bytes).\n\n```fusion\nlet c: char = \'A\';\nlet emoji: char = \'\\u{1F600}\';\n```',
      'str': '**str** - String slice\n\nReference to a UTF-8 string.\n\n```fusion\nlet s: &str = "hello";\n```',
      'String': '**String** - Owned string\n\nGrowable, heap-allocated UTF-8 string.\n\n```fusion\nlet s: String = String::from("hello");\n```',
      'Vec': '**Vec\<T\>** - Dynamic array\n\nGrowable array type.\n\n```fusion\nlet v: Vec<i32> = vec![1, 2, 3];\nlet v: Vec<i32> = Vec::new();\n```',
      'Option': '**Option\<T\>** - Optional type\n\nRepresents optional value: `Some(value)` or `None`.\n\n```fusion\nlet x: Option<i32> = Some(42);\nlet y: Option<i32> = None;\n```\n\nMethods: `is_some()`, `is_none()`, `unwrap()`, `unwrap_or()`, `map()`',
      'Result': '**Result\<T, E\>** - Result type\n\nRepresents success or failure: `Ok(value)` or `Err(error)`.\n\n```fusion\nlet r: Result<i32, String> = Ok(42);\nlet e: Result<i32, String> = Err("error".into());\n```\n\nMethods: `is_ok()`, `is_err()`, `unwrap()`, `unwrap_or()`, `map()`, `and_then()`',
      'Box': '**Box\<T\>** - Heap-allocated value\n\nOwns data on the heap.\n\n```fusion\nlet b: Box<i32> = Box::new(42);\n```',
      'HashMap': '**HashMap\<K, V\>** - Hash map\n\nKey-value store with O(1) lookup.\n\n```fusion\nlet mut map: HashMap<String, i32> = HashMap::new();\nmap.insert("key".into(), 42);\n```',
      // Quantum types
      'QuantumState': '**QuantumState** - Quantum state representation\n\nRepresents the state of a quantum system as a complex vector.',
      'QuantumRegister': '**QuantumRegister** - Quantum register\n\nCollection of qubits for quantum operations.\n\n```fusion\nlet reg: QuantumRegister = QuantumRegister::new(5); // 5-qubit register\n```',
      'QuantumCircuit': '**QuantumCircuit** - Quantum circuit\n\nSequence of quantum gates applied to qubits.\n\n```fusion\nlet circuit: QuantumCircuit = QuantumCircuit::new();\ncircuit.hadamard(0);\ncircuit.cnot(0, 1);\n```',
      'Qubit': '**Qubit** - Quantum bit\n\nBasic unit of quantum information. Can be in superposition.\n\n```fusion\nlet q: Qubit = Qubit::new();\nhadamard(&mut q);\n```',
      'Gate': '**Gate** - Quantum gate\n\nRepresents a unitary quantum operation.',
      'Backend': '**Backend** - Quantum simulation backend\n\nTarget platform for quantum circuit execution (simulator, hardware).',
      // PQC types
      'HybridKey': '**HybridKey** - Hybrid cryptographic key\n\nKey combining classical and post-quantum algorithms.',
      'KemCiphertext': '**KemCiphertext** - KEM ciphertext\n\nCiphertext from key encapsulation mechanism.',
      'PqcSignature': '**PqcSignature** - Post-quantum signature\n\nDigital signature resistant to quantum attacks.',
      'DilithiumKey': '**DilithiumKey** - Dilithium signing key\n\nLattice-based digital signature key (NIST PQC standard).\n\n```fusion\nlet (pk, sk): (DilithiumKey, DilithiumKey) = dilithium_keygen();\n```',
      'KyberKey': '**KyberKey** - Kyber encapsulation key\n\nLattice-based KEM key (NIST PQC standard).\n\n```fusion\nlet (pk, sk): (KyberKey, KyberKey) = kyber_keygen();\n```',
      'NeuralSeal': '**NeuralSeal** - NeuralSeal encryption key\n\nNeural network-based homomorphic encryption key.',
      // AI/ML types
      'Tensor': '**Tensor** - Multi-dimensional array\n\nN-dimensional numerical array for computation.\n\n```fusion\nlet t: Tensor = zeros([2, 3, 4]);\nlet m: Tensor = matmul(a, b);\n```',
      'AutodiffGraph': '**AutodiffGraph** - Automatic differentiation graph\n\nComputation graph that tracks operations for gradient computation.',
      'Model': '**Model** - AI/ML model\n\nNeural network model for training and inference.\n\n```fusion\nlet m: Model = Model::load("model.fus");\nlet result = m.forward(input);\n```',
      'Optimizer': '**Optimizer** - Training optimizer\n\nOptimization algorithm for model training.\n\n```fusion\nlet opt: Optimizer = adam(model.parameters(), lr=0.001);\n```',
      'LossFunction': '**LossFunction** - Loss function\n\nFunction measuring prediction error during training.\n\n```fusion\nlet loss: LossFunction = mse_loss();\n```',
      'Dataset': '**Dataset** - Training dataset\n\nCollection of training examples.\n\n```fusion\nlet data: Dataset = Dataset::from_csv("train.csv");\n```',
      // Runtime types
      'SupernovaRuntime': '**SupernovaRuntime** - Async runtime\n\nHigh-performance asynchronous runtime for Fusion.',
      'CortexEngine': '**CortexEngine** - AI reasoning engine\n\nNeural reasoning and planning engine.',
      'IntentGraph': '**IntentGraph** - Intent graph\n\nGraph structure for AI intent reasoning.',
      'HaftScheduler': '**HaftScheduler** - Task scheduler\n\nHigh-performance work-stealing task scheduler.',
      'Fiber': '**Fiber** - Lightweight concurrency unit\n\nGreen thread for cooperative multitasking.\n\n```fusion\nlet f: Fiber = Fiber::new(|| {\n    // concurrent work\n});\n```',
      // Cloud types
      'KubernetesCluster': '**KubernetesCluster** - K8s cluster\n\nRepresents a Kubernetes cluster endpoint.',
      'Container': '**Container** - Container\n\nDocker/OCI container configuration.\n\n```fusion\nlet c: Container = Container::new("nginx:latest");\n```',
      'Pod': '**Pod** - Kubernetes pod\n\nSmallest deployable unit in Kubernetes.',
      'Service': '**Service** - Network service\n\nExposes an application as a network service.',
      'FaasEndpoint': '**FaasEndpoint** - FaaS endpoint\n\nServerless function endpoint.',
      // Interop types
      'PyModule': '**PyModule** - Python module handle\n\nHandle to an imported Python module.\n\n```fusion\nlet np = py_import numpy;\n```',
      'JsValue': '**JsValue** - JavaScript value\n\nHandle to a JavaScript value.\n\n```fusion\nlet result = js_eval(JSON.stringify(data));\n```',
      'JvmClass': '**JvmClass** - JVM class reference\n\nReference to a Java/JVM class.'
    };

    const doc = typeDocs[word];
    if (!doc) {
      return null;
    }

    const markdown = new MarkdownString(doc);
    return new Hover(markdown);
  }

  private getMemberHover(word: string, document: TextDocument, position: Position): Hover | null {
    const methodDocs: Record<string, string> = {
      'len': '**fn len(&self) -> usize**\n\nReturns the number of elements.\n\n```fusion\nlet v = vec![1, 2, 3];\nassert_eq!(v.len(), 3);\n```',
      'is_empty': '**fn is_empty(&self) -> bool**\n\nReturns `true` if the collection is empty.\n\n```fusion\nlet v: Vec<i32> = vec![];\nassert!(v.is_empty());\n```',
      'push': '**fn push(&mut self, value: T)**\n\nAppends an element to the end.\n\n```fusion\nlet mut v = vec![1, 2];\nv.push(3);\n```',
      'pop': '**fn pop(&mut self) -> Option\<T\>**\n\nRemoves and returns the last element.\n\n```fusion\nlet mut v = vec![1, 2, 3];\nassert_eq!(v.pop(), Some(3));\n```',
      'contains': '**fn contains(&self, value: &T) -> bool**\n\nReturns `true` if element is in collection.\n\n```fusion\nlet v = vec![1, 2, 3];\nassert!(v.contains(&2));\n```',
      'iter': '**fn iter(&self) -> Iter\<T\>**\n\nReturns an iterator over references.\n\n```fusion\nfor item in v.iter() {\n    println!(\"{}\", item);\n}\n```',
      'map': '**fn map\<F, B\>(self, f: F) -> Map\<B\>**\n\nTransforms each element.\n\n```fusion\nlet doubled: Vec<i32> = v.iter().map(|x| x * 2).collect();\n```',
      'filter': '**fn filter\<P\>(self, predicate: P) -> Filter\<T\>**\n\nFilters elements by predicate.\n\n```fusion\nlet evens: Vec<&i32> = v.iter().filter(|&&x| x % 2 == 0).collect();\n```',
      'fold': '**fn fold\<B, F\>(self, init: B, f: F) -> B**\n\nReduces to single value.\n\n```fusion\nlet sum = v.iter().fold(0, |acc, &x| acc + x);\n```',
      'collect': '**fn collect\<C\>(self) -> C**\n\nCollects into a collection type.\n\n```fusion\nlet v: Vec<i32> = (0..5).collect();\n```',
      'clone': '**fn clone(&self) -> Self**\n\nReturns a copy of the value.\n\n```fusion\nlet original = String::from(\"hello\");\nlet cloned = original.clone();\n```',
      'to_string': '**fn to_string(&self) -> String**\n\nConverts to String using Display trait.\n\n```fusion\nlet s = 42.to_string();\n```',
      'unwrap': '**fn unwrap(self) -> T**\n\nReturns contained value or panics.\n\n```fusion\nlet x: Option<i32> = Some(42);\nlet val = x.unwrap(); // 42\n```\n\nUse `unwrap_or()` or pattern matching for safer code.',
      'unwrap_or': '**fn unwrap_or(self, default: T) -> T**\n\nReturns contained value or default.\n\n```fusion\nlet x: Option<i32> = None;\nlet val = x.unwrap_or(0); // 0\n```',
      'expect': '**fn expect(self, msg: &str) -> T**\n\nReturns contained value or panics with message.\n\n```fusion\nlet val = x.expect(\"value should exist\");\n```',
      // QuantumCircuit methods
      'hadamard': '**fn hadamard(&mut self, qubit: usize)**\n\nApplies Hadamard gate to specified qubit.\n\n```fusion\nlet mut circuit = QuantumCircuit::new();\ncircuit.hadamard(0);\n```',
      'cnot': '**fn cnot(&mut self, control: usize, target: usize)**\n\nApplies CNOT gate.\n\n```fusion\ncircuit.cnot(0, 1);\n```',
      'measure_all': '**fn measure_all(&mut self) -> Vec<u8>**\n\nMeasures all qubits in the circuit.',
      // Tensor methods
      'shape': '**fn shape(&self) -> Vec\<usize\>**\n\nReturns tensor dimensions.\n\n```fusion\nlet t = zeros([2, 3]);\nassert_eq!(t.shape(), vec![2, 3]);\n```',
      'transpose': '**fn transpose(&self) -> Tensor**\n\nReturns transposed tensor.',
      'matmul': '**fn matmul(&self, other: &Tensor) -> Tensor**\n\nMatrix multiplication.',
      // Model methods
      'forward': '**fn forward(&self, input: &Tensor) -> Tensor**\n\nRuns forward pass through model.',
      'parameters': '**fn parameters(&self) -> Vec\<Tensor\>**\n\nReturns model trainable parameters.',
      'train': '**fn train(&mut self, data: &Dataset, epochs: u32)**\n\nTrains the model on dataset.'
    };

    const doc = methodDocs[word];
    if (!doc) {
      return null;
    }

    const markdown = new MarkdownString(doc);
    return new Hover(markdown);
  }

  private getPathHover(word: string): Hover | null {
    const moduleDocs: Record<string, string> = {
      'math': '**math** - Mathematical functions\n\nProvides: `abs`, `sqrt`, `pow`, `min`, `max`, `ceil`, `floor`, `round`',
      'quantum': '**quantum** - Quantum computing functions\n\nProvides: `hadamard`, `cnot`, `toffoli`, `phase`, `rotate_x`, `rotate_y`, `rotate_z`, `measure`, `create_qubit`, `entangle`, `superpose`, `pauli_x`, `pauli_y`, `pauli_z`, `swap`, `rx`, `ry`, `rz`, `simulate`, `create_circuit`, `run_circuit`',
      'hybrid': '**hybrid** - Hybrid cryptography functions\n\nProvides: `encrypt`, `decrypt`, `sign`, `verify`, `keygen`, `encapsulate`, `decapsulate`',
      'pqc': '**pqc** - Post-quantum cryptography functions\n\nProvides: `dilithium_keygen`, `dilithium_sign`, `dilithium_verify`, `kyber_keygen`, `kyber_encapsulate`, `kyber_decapsulate`, `neuralseal_encrypt`, `neuralseal_decrypt`',
      'tensor': '**tensor** - Tensor computation functions\n\nProvides: `zeros`, `ones`, `full`, `arange`, `linspace`, `rand`, `randn`, `matmul`, `transpose`, `reshape`, `softmax`, `relu`, `sigmoid`, `tanh`, `mse_loss`, `cross_entropy`, `gradient`, `backward`, `sgd_step`, `adam_step`',
      'autodiff': '**autodiff** - Automatic differentiation\n\nProvides: `grad`, `jacobian`, `hessian`, `vjp`, `jvp`, `tape`, `enable_grad`, `disable_grad`',
      'supernova': '**supernova** - Async runtime\n\nProvides: `spawn`, `join`, `yield_now`, `sleep`, `block_on`, `spawn_blocking`, `task_local`, `JoinHandle`',
      'cortex': '**cortex** - AI reasoning engine\n\nProvides: `intent`, `plan`, `execute`, `observe`, `adapt`, `reason`, `infer`, `cluster`, `topology`, `shard`',
      'haft': '**haft** - Task scheduler\n\nProvides: `schedule`, `dispatch`, `prioritize`, `preempt`, `yield`, `queue_depth`, `worker_count`, `set_affinity`',
      'fiber': '**fiber** - Lightweight concurrency\n\nProvides: `new`, `spawn_fiber`, `block_on_fiber`, `fiber_id`, `fiber_local`, `yield_fiber`',
      'kubernetes': '**kubernetes** - Kubernetes management\n\nProvides: `deploy_pod`, `create_service`, `list_pods`, `scale_deployment`, `delete_pod`, `get_logs`, `apply_yaml`',
      'faas': '**faas** - Function-as-a-Service\n\nProvides: `invoke`, `deploy_function`, `list_functions`, `delete_function`, `get_invocation_logs`, `set_memory_limit`, `set_timeout`',
      'collections': '**collections** - Collection types\n\nProvides: `vec`, `hashmap`, `btreemap`, `btree_set`, `linked_list`, `vec_deque`',
      'io': '**io** - Input/Output functions\n\nProvides: `print`, `println`, `eprint`, `eprintln`, `format`, `read_line`, `stdin`, `stdout`, `stderr`',
      'string': '**string** - String functions\n\nProvides: `len`, `push`, `pop`, `contains`, `starts_with`, `ends_with`, `split`, `trim`, `to_upper`, `to_lower`, `replace`, `parse`',
      'convert': '**convert** - Type conversion\n\nProvides: `into`, `from`, `try_into`, `try_from`, `as_ref`, `to_string`, `as_str`',
      'iter': '**iter** - Iterator functions\n\nProvides: `map`, `filter`, `fold`, `reduce`, `collect`, `enumerate`, `zip`, `chain`, `take`, `skip`, `any`, `all`, `find`, `for_each`, `sum`, `count`, `max`, `min`',
      'option': '**option** - Option functions\n\nProvides: `is_some`, `is_none`, `unwrap`, `unwrap_or`, `unwrap_or_else`, `map`, `and_then`, `ok_or`, `ok_or_else`',
      'result': '**result** - Result functions\n\nProvides: `is_ok`, `is_err`, `unwrap`, `unwrap_or`, `unwrap_or_else`, `map`, `map_err`, `and_then`, `ok`, `err`, `expect`',
      'interop': '**interop** - Language interop\n\nProvides: `py_import`, `js_eval`, `jvm_call`, `py_call`, `js_call`, `jvm_new`, `to_python`, `to_javascript`, `to_java`, `from_python`, `from_javascript`, `from_java`',
      'crypto': '**crypto** - Cryptographic functions\n\nProvides: `gen_keypair`, `sign_message`, `verify_signature`, `encrypt_data`, `decrypt_data`, `derive_shared_secret`, `hash_sha256`, `hash_sha512`, `hmac_sign`, `hmac_verify`, `aes_encrypt`, `aes_decrypt`'
    };

    const doc = moduleDocs[word];
    if (!doc) {
      return null;
    }

    const markdown = new MarkdownString(doc);
    return new Hover(markdown);
  }

  private getMacroHover(word: string): Hover | null {
    const macroDocs: Record<string, string> = {
      'println!': '**println!** - Print with newline\n\n```fusion\nprintln!(\"Hello, {}!\", name);\nprintln!(\"Value: {:?}\", value);\n```',
      'print!': '**print!** - Print without newline\n\n```fusion\nprint!(\"Processing...\");\n```',
      'format!': '**format!** - Format to String\n\n```fusion\nlet s = format!(\"Hello, {}!\", name);\n```',
      'vec!': '**vec!** - Create vector\n\n```fusion\nlet v = vec![1, 2, 3];\nlet v = vec![0; 10]; // 10 zeros\n```',
      'panic!': '**panic!** - Panic with message\n\n```fusion\npanic!(\"something went wrong\");\npanic!(\"value was {}\", x);\n```',
      'assert!': '**assert!** - Assert condition\n\n```fusion\nassert!(x > 0);\nassert!(x > 0, \"x must be positive\");\n```',
      'assert_eq!': '**assert_eq!** - Assert equality\n\n```fusion\nassert_eq!(a, b);\nassert_eq!(a, b, \"values not equal\");\n```',
      'todo!': '**todo!** - Mark unimplemented\n\nPanics with \"not yet implemented\".',
      'unreachable!': '**unreachable!** - Mark unreachable code\n\nPanics with \"entered unreachable code\".',
      'stringify!': '**stringify!** - Convert to string literal\n\n```fusion\nlet s = stringify!(hello); // \"hello\"\n```'
    };

    if (word.endsWith('!')) {
      const doc = macroDocs[word];
      if (doc) {
        return new Hover(new MarkdownString(doc));
      }
    }

    return null;
  }

  private getSymbolHover(word: string, document: TextDocument, position: Position): Hover | null {
    const text = document.getText();
    const fnRegex = new RegExp(`pub\\s+(?:async\\s+)?fn\\s+${word}\\s*(?:<[^>]*>)?\\s*\\(([^)]*)\\)(?:\\s*->\\s*(\\S+))?`, 'g');
    const match = fnRegex.exec(text);

    if (match) {
      const params = match[1] || '';
      const returnType = match[2] || '()';
      const markdown = new MarkdownString(
        `**fn ${word}**(${params}) -> ${returnType}\n\nFunction defined in this file.`
      );
      return new Hover(markdown);
    }

    return null;
  }
}
