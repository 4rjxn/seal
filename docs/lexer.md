# Lexer Architecture & Design Guide

This document defines the architectural specification of the Seal Lexer, its place in the pipeline, and the critical edge cases to watch out for.

---

## 1. Architectural Role & Responsibilities

The Lexer (tokenizer) represents the first phase of the shell's frontend. Its sole purpose is to convert an arbitrary stream of UTF-8 characters (`&str`) into a discrete sequence of semantic tokens (`Token`).

```text
[ Raw String Input ] ──▶ [ Lexer (Iterator) ] ──▶ [ Stream of Tokens ]
```

### Key Architectural Boundaries:
- **Zero Knowledge of Execution:** The lexer does not know what commands exist, what environment variables are set, or whether a path is valid.
- **No Expansion:** The lexer does **not** evaluate `$VAR`, `~`, or globs (`*.txt`). It only records whether a token was enclosed in quotes so downstream stages know whether expansion is permitted.
- **Pure & Deterministic:** Given the exact same string slice, the lexer will always produce the exact same sequence of tokens without side effects.

---

## 2. Core Architectural Model: Token-Pull (`Iterator`)

### Why "Token-Pull" Replaces "Character-Push"
* **Character-Push (Anti-Pattern):** An external loop feeds characters into a state machine one by one. This breaks down when operators span multiple characters (`>>`, `&&`, `2>`) or when an operator immediately follows a word without spaces (`hello>file`). Emitting two tokens from a single character forces messy state accumulators and vector-flushing hacks.
* **Token-Pull (The Pattern):** The consumer (the Parser or REPL Validator) calls `.next()`. The Lexer autonomously reads ahead as many characters as needed from a `Peekable` iterator to produce **exactly one clean token**.

### Token Hierarchy
Tokens are categorized into three distinct syntactic classes:
1. **Words (`Token::Word`):** Commands, arguments, file paths, and literal text. Tracks whether the word contained quoting (`quoted: bool`) to govern downstream variable expansion.
2. **Control Operators:** Sequential and logical flow delimiters (`|`, `&&`, `||`, `&`, `;`).
3. **Redirection Operators:** I/O routing descriptors (`>`, `>>`, `2>`, `2>>`, `<`).

---

## 3. Specific Things to Keep an Eye On (Gotchas & Pitfalls)

When implementing or modifying the lexer, watch out for these subtle edge cases:

### 1. The "Sticky Quoted" Flag Hazard
* **The Pitfall:** If `quoted` state is stored as a global boolean on the Lexer struct and not strictly cleared whenever a word finishes, the quote flag will "stick" to all subsequent words on the command line.
* **Impact:** Once a user types a quoted argument (`echo "hello" $USER`), every following argument is marked as quoted, silently disabling environment variable expansion, tilde expansion, and globbing for the rest of the command line.
* **Watch Out:** The `quoted` flag must be scoped **strictly to the lifetime of an individual word**.

### 2. Metacharacters Without Whitespace
* **The Pitfall:** Treating operators (`|`, `>`, `<`, `&`) as ordinary word characters unless separated by spaces.
* **Impact:** `ls|grep a` is lexed as a single word `"ls|grep"`. `echo hi>out.txt` is lexed as `"echo"` and `"hi>out.txt"`.
* **Watch Out:** Unquoted metacharacters are strict word delimiters. When lexing a word, encountering any unquoted metacharacter must immediately terminate the current word and leave the operator character unconsumed for the next token pull.

### 3. Quoted Metacharacters Must Remain Words
* **The Pitfall:** Matching operator strings like `"|"` or `">"` without checking if they were enclosed in quotes.
* **Impact:** Commands like `grep "|"` or `echo ">"` get hijacked into pipe and redirect tokens, breaking command arguments.
* **Watch Out:** An operator is *only* an operator if it was encountered outside of quotes. If quotes were present, it must always be treated as a literal `Token::Word`.

### 4. Distinguishing File Descriptors from Word Digits
* **The Pitfall:** Misinterpreting numbers preceding a redirect.
* **Rules to maintain:**
  - `echo hello 2> err.log` $\rightarrow$ `2>` is an `OutputErr` redirection.
  - `python3 2 > out.txt` $\rightarrow$ `2` is an argument, `>` is an `Output` redirection.
  - `cat file2>out.txt` $\rightarrow$ `file2` is a word, `>` is an `Output` redirection.
* **Watch Out:** A digit (`1` or `2`) is only part of a redirect operator if it is directly adjacent to `>` with no whitespace between them, AND preceded by whitespace or at the start of a command.

### 5. Quoting Contexts: Single vs. Double vs. Escape
* **Single Quotes (`'...'`):** Everything inside is 100% literal. Backslashes have no special escaping power. If EOF is reached before the closing quote, it is an `UnclosedQuote('\'')` error.
* **Double Quotes (`"..."`):** Allows embedded spaces and metacharacters. Backslashes *do* escape `\"`, `\\`, and `\$`. Unclosed double quotes produce an `UnclosedQuote('"')` error.
* **Backslash Outside Quotes (`\`):** Escapes the single next character literally (e.g. `\ ` preserves a literal space in a filename). If EOF is reached immediately after `\`, it is an `UnexpectedEof` error.

### 6. REPL Validation (Multiline Editing Support)
* **The Need:** Rustyline's `Validator` needs the lexer to tell whether a command line is incomplete.
* **Watch Out:** 
  - An `UnclosedQuote` or trailing `\` should not be treated as a fatal error by the REPL; it signals that the user pressed `Enter` while intending to continue typing on a new line (continuation prompt `...> `).
  - Trailing operators like `|`, `&&`, and `||` also represent incomplete input that requires multiline continuation.
