# Compilation Techniques Project

**Student:** Mohamad Said Monawar  
**Course:** Compilation Techniques  
**Programming Language:** Rust  
**Academic Year:** 2025–2026  

---

## 1. Project Overview

This project implements a sequence of compiler-related techniques for two small languages: MiniImp and MiniFun.

The project includes:

- A lexer and recursive-descent parser for MiniImp
- An operational-semantics interpreter for MiniImp
- A lexer and recursive-descent parser for MiniFun
- A closure-based interpreter for MiniFun
- Annotated static type checking for MiniFun
- Polymorphic type inference for MiniFun
- Control Flow Graph construction for MiniImp
- Defined-variable analysis
- Live-variable analysis
- Reaching-definitions analysis
- Possibly undefined-variable checking
- Constant folding
- Constant propagation
- Dead-store elimination
- An optimization pipeline
- LLVM IR generation
- Conversion to SSA form using LLVM `mem2reg`
- Native object-code generation
- Linking with a C wrapper
- Execution of the final compiled program

The completed processing flow is:

```text
Source Code
    ↓
Lexer
    ↓
Tokens
    ↓
Parser
    ↓
Abstract Syntax Tree
    ↓
Evaluation / Type Checking
    ↓
Control Flow Graph
    ↓
Data-flow Analysis
    ↓
Optimizations
    ↓
LLVM IR
    ↓
SSA Form
    ↓
Native Object File
    ↓
Executable Program
```

The MiniImp and MiniFun lexers and parsers were added after the completion of the main eight project fragments. They complete the frontend of the project and allow textual programs to be transformed automatically into the existing abstract syntax trees.

---

## 2. Implemented Fragments

### Fragment 1 — MiniImp Interpreter

This fragment implements:

- The MiniImp abstract syntax tree
- Arithmetic expressions
- Boolean expressions
- Commands
- The runtime memory
- Operational-semantics evaluation
- MiniImp lexical analysis
- MiniImp recursive-descent parsing

### Fragment 2 — MiniFun Interpreter

This fragment implements:

- The MiniFun abstract syntax tree
- Integer and Boolean values
- Functions and applications
- Closures
- Static scoping
- Let-bindings
- Recursive `letfun` definitions
- MiniFun lexical analysis
- MiniFun recursive-descent parsing

### Fragment 3 — Annotated Type Checking

This fragment implements:

- Integer, Boolean, and function types
- Type environments
- Annotated functions
- Annotated recursive functions
- Static type checking
- Descriptive type errors

### Fragment 4 — Polymorphic Type Inference

This fragment implements:

- Monotypes
- Polytypes
- Type substitutions
- Fresh type-variable generation
- Instantiation
- Generalization
- Unification
- Occurs checking
- Polymorphic let-bindings

### Fragment 5 — Control Flow Graph Construction

This fragment implements:

- CFG blocks
- Minimal basic blocks
- Entry and exit blocks
- Conditional branches
- Loop back-edges
- Successor and predecessor operations
- Graphviz DOT generation

### Fragment 6 — Data-flow Analysis

This fragment implements:

- Defined-variable analysis
- Live-variable analysis
- Reaching-definitions analysis
- Generic CFG annotations
- IN and OUT sets
- Fixed-point iteration

### Fragment 7 — Compiler Optimizations

This fragment implements:

- Possibly undefined-variable detection
- Constant folding
- Constant propagation
- Dead-store elimination
- Configurable optimization passes
- Repeated optimization until a fixed point

### Fragment 8 — LLVM IR Generation

This fragment implements:

- Textual LLVM IR generation
- Stack allocation for mutable variables
- Arithmetic and Boolean LLVM instructions
- Conditional and unconditional branches
- Unique LLVM temporary names
- SSA conversion using `mem2reg`
- Native object-code generation
- Linking with a C wrapper

---

## 3. Project Structure

The main project structure is:

```text
compilation_project/
├── Cargo.toml
├── Cargo.lock
├── README.md
├── wrapper.c
├── src/
│   ├── main.rs
│   ├── miniimp/
│   │   ├── ast.rs
│   │   ├── lexer.rs
│   │   ├── parser.rs
│   │   ├── runtime.rs
│   │   ├── eval.rs
│   │   ├── cfg.rs
│   │   ├── dataflow.rs
│   │   ├── optimizations.rs
│   │   ├── llvm.rs
│   │   └── mod.rs
│   └── minifun/
│       ├── ast.rs
│       ├── lexer.rs
│       ├── parser.rs
│       ├── runtime.rs
│       ├── eval.rs
│       ├── types.rs
│       ├── typecheck.rs
│       ├── inference.rs
│       └── mod.rs
```

The project may also generate the following files during execution:

```text
cfg.dot
cfg_defined.dot
cfg_live.dot
cfg_reaching.dot
cfg_optimized.dot
program.ll
program_opt.ll
program.o
program
```

---

## 4. Requirements

### Rust-side requirements

The Rust project requires:

- Rust
- Cargo

### LLVM requirements

The native compilation stage requires:

- LLVM
- `opt`
- `llc`
- Clang

### Recommended environment on Windows

The Rust commands can be executed from PowerShell, Command Prompt, VS Code, or WSL.

The LLVM commands should be executed inside WSL or another Linux environment if the LLVM tools are not installed directly on Windows.

---

## 5. Verify the Installed Tools

Check Rust and Cargo:

```bash
rustc --version
cargo --version
```

Check LLVM and Clang:

```bash
opt --version
llc --version
clang --version
```

Each command should print the installed version.

If `opt`, `llc`, or `clang` is not recognized in Windows PowerShell, enter WSL and run the commands there.

---

## 6. Open the Project Directory

Open a terminal in the directory containing `Cargo.toml`.

For example:

```bash
cd /path/to/compilation_project
```

When the project is stored on the Windows `C:` drive and WSL is being used, the path may look like:

```bash
cd /mnt/c/Users/USERNAME/compilation_project
```

Replace `USERNAME` and the remaining path with the actual location of the project.

Example:

```bash
cd /mnt/c/Users/mhmsa/compilation_project
```

Verify that the correct directory is open:

```bash
ls
```

The output should include files such as:

```text
Cargo.toml
Cargo.lock
src
wrapper.c
README.md
```

---

## 7. Format the Rust Code

Before building, format the source code:

```bash
cargo fmt
```

This command automatically applies the standard Rust formatting style.

To check formatting without modifying files, use:

```bash
cargo fmt --check
```

---

## 8. Build the Project

Run:

```bash
cargo build
```

A successful build finishes without compilation errors.

Rust may display warnings about unused helper functions, methods, or enum variants. Warnings do not prevent the project from compiling, but compilation errors must be corrected.

A typical successful result contains:

```text
Finished `dev` profile
```

---

## 9. Run the Unit Tests

Run:

```bash
cargo test
```

This executes tests for:

- MiniImp lexical analysis
- MiniImp parsing
- MiniImp precedence and parentheses
- MiniImp assignments
- MiniImp conditionals
- MiniImp loops
- MiniFun lexical analysis
- MiniFun parsing
- MiniFun function application
- MiniFun recursive functions
- MiniImp and MiniFun evaluation
- Annotated type checking
- Polymorphic type inference
- CFG construction
- Defined-variable analysis
- Live-variable analysis
- Reaching-definitions analysis
- Undefined-variable checking
- Constant folding
- Constant propagation
- Dead-store elimination
- Optimization-pipeline execution
- LLVM IR generation

A successful test execution should end with:

```text
test result: ok
```

The number of failed tests must be zero.

For example:

```text
test result: ok. 30 passed; 0 failed
```

The exact number of tests may change if more tests are added, but all tests must pass.

---

## 10. Run the Complete Project Demonstration

Run:

```bash
cargo run
```

The main program executes an integrated demonstration of the complete project.

It performs the following operations:

1. Tokenizes and parses a MiniImp source program.
2. Tests arithmetic precedence.
3. Tests parentheses.
4. Evaluates the parsed MiniImp program.
5. Tokenizes and parses a recursive MiniFun factorial program.
6. Evaluates the factorial program.
7. Performs annotated MiniFun type checking.
8. Performs polymorphic MiniFun type inference.
9. Constructs a MiniImp Control Flow Graph.
10. Runs defined-variable analysis.
11. Runs live-variable analysis.
12. Runs reaching-definitions analysis.
13. Checks for possibly undefined variables.
14. Runs the optimization pipeline until a fixed point.
15. Generates LLVM IR in `program.ll`.

The important output should look similar to:

```text
========== COMPLETE PROJECT TEST ==========

FRAGMENT 1 - MiniImp lexer, parser, evaluator
Tokens produced: 32
Precedence test 2 + 3 * 4: 14 (expected 14)
Parentheses test (2 + 3) * 4: 20 (expected 20)
Evaluation with input 6: 14 (expected 14)
Status: PASS

FRAGMENT 2 - MiniFun lexer, parser, evaluator
Tokens produced: 28
factorial(5): 120 (expected 120)
Status: PASS

FRAGMENT 3 - MiniFun annotated type checking
Type of factorial program: Int (expected Int)
Status: PASS

FRAGMENT 4 - Polymorphic type inference
Inferred result type: Bool (expected Bool)
Status: PASS

FRAGMENT 5 - CFG construction
CFG blocks: 5 (expected 5)
Generated: cfg.dot
Status: PASS

FRAGMENT 6 - Data-flow analyses
Defined variables analysis: completed
Live variables analysis: completed
Reaching definitions analysis: completed
Generated: cfg_defined.dot, cfg_live.dot, cfg_reaching.dot
Status: PASS

FRAGMENT 7 - Undefined-variable checking and optimizations
Undefined-variable check: passed
Reached fixed point: true
Optimized output assignment: out := 65
Generated: cfg_optimized.dot
Status: PASS

FRAGMENT 8 - LLVM IR generation
Generated: program.ll
Expected native result: 14
Status: PASS

========== ALL RUST-SIDE TESTS PASSED ==========
```

The exact token counts, number of optimization rounds, and number of changed pass executions may vary if the source examples or implementation are modified.

The essential conditions are:

```text
Status: PASS
Reached fixed point: true
========== ALL RUST-SIDE TESTS PASSED ==========
```

---

## 11. Files Generated by `cargo run`

Running:

```bash
cargo run
```

generates:

```text
cfg.dot
cfg_defined.dot
cfg_live.dot
cfg_reaching.dot
cfg_optimized.dot
program.ll
```

### `cfg.dot`

Contains the original MiniImp Control Flow Graph.

### `cfg_defined.dot`

Contains the CFG annotated with defined-variable analysis results.

### `cfg_live.dot`

Contains the CFG annotated with live-variable analysis results.

### `cfg_reaching.dot`

Contains the CFG annotated with reaching-definitions analysis results.

### `cfg_optimized.dot`

Contains the optimized CFG.

### `program.ll`

Contains the initial memory-based LLVM IR generated by the Rust backend.

---

## 12. Optional Graphviz Visualization

The `.dot` files can be converted into images using Graphviz.

If Graphviz is installed, run:

```bash
dot -Tpng cfg.dot -o cfg.png
dot -Tpng cfg_defined.dot -o cfg_defined.png
dot -Tpng cfg_live.dot -o cfg_live.png
dot -Tpng cfg_reaching.dot -o cfg_reaching.png
dot -Tpng cfg_optimized.dot -o cfg_optimized.png
```

This produces PNG images that can be opened using an image viewer.

Graphviz is not required for the Rust project or LLVM compilation. It is only needed to visualize the generated CFG files.

---

## 13. Convert LLVM IR to SSA Form

The generated `program.ll` initially represents MiniImp variables using:

- `alloca`
- `load`
- `store`

Run the LLVM `mem2reg` pass:

```bash
opt -passes="mem2reg" program.ll -S -o program_opt.ll
```

This generates:

```text
program_opt.ll
```

The `mem2reg` pass promotes eligible stack variables to SSA registers.

At control-flow merge points or loop headers, LLVM may insert phi instructions automatically.

To search for phi instructions, run:

```bash
grep -n "phi" program_opt.ll
```

An example result may look like:

```text
9:  %var1.0 = phi i64 [ %tmp2, %block1 ], [ %tmp4, %block2 ]
```

The exact names and line numbers may vary.

---

## 14. Generate the Native Object File

Translate the optimized LLVM IR into a native object file:

```bash
llc -filetype=obj program_opt.ll -o program.o
```

This generates:

```text
program.o
```

The object file contains native machine code but is not yet a complete executable.

---

## 15. Link with the C Wrapper

Run:

```bash
clang wrapper.c program.o -o program
```

This links:

- The generated object file `program.o`
- The C wrapper `wrapper.c`

The result is the executable:

```text
program
```

The C wrapper reads an integer command-line argument, calls the generated LLVM function, and prints its returned integer result.

---

## 16. Execute the Native Program

Run:

```bash
./program 6
```

Expected output:

```text
14
```

This confirms that the program completed the full compilation process:

```text
MiniImp Source Code
    ↓
Lexer
    ↓
Tokens
    ↓
Parser
    ↓
AST
    ↓
CFG
    ↓
LLVM IR
    ↓
SSA Form
    ↓
Object File
    ↓
Native Executable
```

---

## 17. Complete Verification Sequence

The complete sequence for formatting, building, testing, running, optimizing, compiling, linking, and executing the project is:

```bash
cargo fmt
cargo build
cargo test
cargo run

opt -passes="mem2reg" program.ll -S -o program_opt.ll
llc -filetype=obj program_opt.ll -o program.o
clang wrapper.c program.o -o program
./program 6
```

The final native output should be:

```text
14
```

---

## 18. Running the Project in WSL

From Windows PowerShell, start WSL:

```powershell
wsl
```

Navigate to the project directory:

```bash
cd /mnt/c/Users/USERNAME/compilation_project
```

Verify the project files:

```bash
ls
```

Then run:

```bash
cargo fmt
cargo build
cargo test
cargo run
```

After `program.ll` is generated, run:

```bash
opt -passes="mem2reg" program.ll -S -o program_opt.ll
llc -filetype=obj program_opt.ll -o program.o
clang wrapper.c program.o -o program
./program 6
```

Expected output:

```text
14
```

---

## 19. Using Visual Studio Code

The project can be opened in Visual Studio Code.

From the project directory, run:

```bash
code .
```

When working inside WSL, the VS Code WSL extension is recommended.

The integrated VS Code terminal can be used to execute all Rust and LLVM commands.

Ensure that the terminal is opened in the project root directory containing `Cargo.toml`.

---

## 20. Troubleshooting

### 20.1 `cargo` is not recognized

Install the Rust toolchain using Rustup, then restart the terminal.

Verify the installation:

```bash
rustc --version
cargo --version
```

### 20.2 `opt: command not found`

Install LLVM and Clang in Ubuntu or WSL:

```bash
sudo apt update
sudo apt install llvm clang
```

Then verify:

```bash
opt --version
llc --version
clang --version
```

### 20.3 LLVM commands are unavailable in PowerShell

Start WSL:

```powershell
wsl
```

Navigate to the project:

```bash
cd /mnt/c/Users/USERNAME/compilation_project
```

Run the LLVM commands inside WSL.

### 20.4 `cargo build` fails

Clean previous build files:

```bash
cargo clean
cargo build
```

If the build still fails, read the first compiler error carefully. Later errors may be consequences of the first one.

### 20.5 A parser test fails

Check the textual source syntax.

For MiniImp, verify:

- Input declaration
- Output declaration
- Semicolons
- `:=` assignments
- Parentheses
- Braces
- `then`
- `else`
- `do`

Example:

```text
input x;
output y;

y := x;

if y < 10 then {
    y := y + 8;
} else {
    y := y - 2;
}
```

For MiniFun, verify:

- Function parameter annotations
- Recursive return-type annotations
- Parentheses
- `->`
- `=`
- `in`
- `then`
- `else`

Example:

```text
letfun fact(n: Int): Int =
    if n < 1 then
        1
    else
        n * fact (n - 1)
in
    fact 5
```

### 20.6 `program.ll` does not exist

Run:

```bash
cargo run
```

The Rust program must complete successfully before the LLVM commands are executed.

### 20.7 `program_opt.ll` does not exist

Run:

```bash
opt -passes="mem2reg" program.ll -S -o program_opt.ll
```

Check that `program.ll` exists in the current directory.

### 20.8 `wrapper.c` produces C errors

Verify that `wrapper.c` contains C code and was not accidentally replaced with Rust source code.

The wrapper should declare and call the LLVM-generated function named `func`.

### 20.9 Permission denied when running `./program`

Run:

```bash
chmod +x program
./program 6
```

Normally Clang creates the executable with the required permissions automatically.

### 20.10 The output is not `14`

Verify that:

1. `cargo run` regenerated `program.ll`.
2. `opt` was run on the latest `program.ll`.
3. `llc` was run on the latest `program_opt.ll`.
4. Clang linked the latest `program.o`.
5. The executable was run with input `6`.

Repeat the complete native sequence:

```bash
opt -passes="mem2reg" program.ll -S -o program_opt.ll
llc -filetype=obj program_opt.ll -o program.o
clang wrapper.c program.o -o program
./program 6
```

---

## 21. Expected Final Results

The project is working correctly when all of the following conditions are satisfied:

- `cargo build` completes successfully.
- `cargo test` reports zero failed tests.
- `cargo run` displays `Status: PASS` for all fragments.
- The optimization pipeline reports `Reached fixed point: true`.
- The final Rust-side line reports:

```text
========== ALL RUST-SIDE TESTS PASSED ==========
```

- `program.ll` is generated.
- `program_opt.ll` is generated.
- `program.o` is generated.
- The executable `program` is generated.
- Running `./program 6` prints:

```text
14
```

---

## 22. AI Usage Statement

During the development of this project, AI-based tools were used as supplementary aids for selected implementation and documentation tasks.

AI assistance was used for:

- Generating initial code templates and portions of auxiliary components, including parts of the MiniImp and MiniFun lexer and parser implementations.
- Suggesting Rust syntax, error-handling approaches, and standard-library usage.
- Reviewing code organization and suggesting improvements to readability and modularity.
- Assisting with the organization, wording, and formatting of the project report and README documentation.
- Suggesting test cases for lexical analysis, parsing, operator precedence, parentheses, evaluation, type checking, data-flow analysis, optimizations, and LLVM generation.

All AI-generated suggestions were reviewed, adapted, compiled, tested, and verified before being incorporated into the final implementation.

The final design decisions, implementation choices, integration work, debugging activities, and validation of correctness were performed by the student based on the course material and project requirements.

The student remains responsible for understanding, correctness, and the final content of the submitted implementation and documentation.

---

## 23. Final Notes

The MiniImp and MiniFun parsers were intentionally implemented after the main eight fragments because the remaining compiler components operate on abstract syntax trees and could therefore be developed and tested using manually constructed AST values.

After the parsers were added, the existing evaluator, typechecker, CFG generator, data-flow analyses, optimization passes, and LLVM backend continued to operate on the same AST structures without requiring a redesign.

The final project therefore supports both:

- Manually constructed Rust AST values
- Textual MiniImp and MiniFun source programs processed by the new lexers and parsers