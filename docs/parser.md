# Parser Architecture & AST Design Guide

This document defines the architectural specification of the Seal Parser, the structure of the Abstract Syntax Tree (AST), and the critical edge cases to watch out for.

---

## 1. Architectural Role & Responsibilities

The Parser is the second phase of the shell's frontend. It consumes tokens emitted by the Lexer and constructs a typed, validated **Abstract Syntax Tree (AST)**.

```text
[ Tokens ] ──▶ [ Recursive Descent Parser ] ──▶ [ Abstract Syntax Tree (AST) ]
```

### Key Architectural Boundaries:
- **Zero Runtime State Dependency:** The parser has no access to `ShellState`, environment variables, or process tables. It is a completely pure function.
- **Syntax Validation Only:** The parser validates the grammatical structure of commands, pipes, and redirects. It does **not** check if binaries exist on disk or expand globs.
- **Fail Fast with Structured Errors:** Any syntax error (e.g., unexpected token, missing redirect target) must abort parsing immediately with a typed `ParseError` enum rather than printing to stderr.

---

## 2. AST Hierarchy & Precedence Model

The AST models the full hierarchy of shell control flow:

```text
AstNode (Root)
 ├── Sequence ( ; ) ─────────────── Level 1: Command lists executed sequentially
 ├── Background ( & ) ───────────── Level 2: Entire preceding chain runs in background
 ├── And ( && ) / Or ( || ) ─────── Level 3: Conditional short-circuit evaluation
 └── Pipeline ( | ) ─────────────── Level 4: Data streaming between commands
      └── Command ───────────────── Level 5: Binary + Arguments + Redirections
```

### The Precedence Contract:
1. **Separators (`;`, `&`) bind the loosest:** They terminate complete commands or compound chains.
2. **Logical Chains (`&&`, `||`) bind next:** They connect pipelines conditionally.
3. **Pipes (`|`) bind next:** They stream stdout from one command to stdin of another.
4. **Commands & Redirections bind tightest:** Words and redirection operators combine to form a single executable unit.

---

## 3. Specific Things to Keep an Eye On (Gotchas & Pitfalls)

When implementing or modifying the parser, watch out for these subtle edge cases:

### 1. The Background Operator (`&`) Precedence Trap
* **The Pitfall:** Treating `&` as a flag on an individual `Command` or `Pipeline` rather than wrapping the compound expression.
* **The Example:** `cat a && cat b &`
* **What it means:** In POSIX shell grammar, `&` has the same list-level precedence as `;`. It terminates the *entire* logical chain `(cat a && cat b)` and runs that entire compound job in the background.
* **Watch Out:** The parser must parse the entire `and_or` expression (`cat a && cat b`) first, and then wrap that entire subtree in `AstNode::Background`.

### 2. Free-Floating Redirections
* **The Pitfall:** Assuming redirects always appear at the very end of a command.
* **The Example:** `> output.txt ls -lah` or `grep "foo" < input.txt -i`
* **Shell Rule:** In POSIX shells, redirections can appear *anywhere* in the command's argument list—at the beginning, in the middle, or at the end.
* **Watch Out:** When parsing a command, loop through all incoming words and redirects together. Collect arguments into `args` and redirects into `redirects`, regardless of the order they appear. The first non-redirect word becomes `program`.

### 3. Missing Redirection Targets
* **The Pitfall:** Hitting a redirect token (`>`, `>>`, `2>`, `<`) at the end of input or followed immediately by another operator (`echo hi > | grep foo`).
* **Watch Out:** Every redirection operator *must* be followed by a valid `Token::Word`. If the next token is an operator or EOF, parsing must immediately fail with `ParseError::MissingRedirectTarget`.

### 4. Empty Pipeline Stages
* **The Pitfall:** Allowing naked pipes like `ls | | grep` or trailing pipes `ls |` without arguments.
* **Watch Out:** 
  - `ls | | grep` is a hard syntax error (`ParseError::EmptyPipeline`).
  - A trailing pipe at the end of input (e.g. `cat file |`) should be detected as incomplete input by the REPL validator so it can prompt for the next line instead of failing.

### 5. Keeping Expansion Out of the Parser
* **The Pitfall:** Passing `ShellState` into `parse_arg` to expand `$VAR`, `~`, or globs during parsing.
* **Why it breaks:**
  - It prevents pure unit testing.
  - Glob expansion depends on the current working directory at *execution time*, not parse time.
  - Variable expansion can change command boundaries or produce multiple arguments.
* **Watch Out:** Keep parsing pure. Emit the AST with unexpanded literal words. A separate, dedicated **Expansion Stage** evaluates variables and globs immediately before execution.
